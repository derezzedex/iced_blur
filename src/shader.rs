use iced_core::Rectangle;
use iced_core::Size;
use iced_core::mouse;
use iced_widget::renderer::wgpu::wgpu;
use iced_widget::renderer::wgpu::wgpu::util::DeviceExt;
use iced_widget::shader;

#[derive(Debug)]
pub struct Shader(u32);

impl Shader {
    const MAX_BLUR_RADIUS: u32 = 16;
    pub fn new(radius: u32) -> Self {
        Self(radius)
    }
}

impl<Message> shader::Program<Message> for Shader {
    type State = ();
    type Primitive = Self;

    fn draw(&self, _state: &Self::State, _cursor: mouse::Cursor, _bounds: Rectangle) -> Self {
        Self(self.0)
    }
}

impl shader::Primitive for Shader {
    fn prepare(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        bounds: &Rectangle,
        viewport: &shader::Viewport,
    ) {
        let size = Size::new(
            (bounds.width * viewport.scale_factor() as f32).round() as u32,
            (bounds.height * viewport.scale_factor() as f32).round() as u32,
        );
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, size, format));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();
        pipeline.update(device, size);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        storage.get::<Pipeline>().unwrap().render(
            encoder,
            target.texture(),
            target,
            clip_bounds,
            self.0,
        );
    }
}

pub struct Pipeline {
    offset_alignment: u32,
    downscale_pipeline: wgpu::RenderPipeline,
    upscale_pipeline: wgpu::RenderPipeline,
    texel_bind_group: wgpu::BindGroup,
    textures: [Texture; 2],
    sampler: wgpu::Sampler,
}

impl Pipeline {
    fn new(device: &wgpu::Device, size: Size<u32>, format: wgpu::TextureFormat) -> Self {
        let offset_alignment = device.limits().min_uniform_buffer_offset_alignment;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_blur sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        #[repr(C, align(256))]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct Level {
            size: u32,
            _pad: [u32; 63],
        }

        let texel_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced_blur texel bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<Level>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            }],
        });

        let levels = (0..=Shader::MAX_BLUR_RADIUS)
            .map(|i| Level {
                size: 2u32.pow(i),
                _pad: bytemuck::Zeroable::zeroed(),
            })
            .collect::<Vec<_>>();
        let texel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("texel size buffer"),
            contents: bytemuck::cast_slice(&levels),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let texel_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_blur texel bind group"),
            layout: &texel_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    size: wgpu::BufferSize::new(std::mem::size_of::<Level>() as u64),
                    ..texel_buffer.as_entire_buffer_binding()
                }),
            }],
        });

        let size = Size::new(
            size.width.next_power_of_two(),
            size.height.next_power_of_two(),
        );
        let texture1 = Texture::new(device, size, format, &sampler);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_blur downsample render pipeline layout"),
            bind_group_layouts: &[&texture1.bind_group_layout, &texel_layout],
            push_constant_ranges: &[],
        });

        let downscale_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_blur downsample shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/downsample.wgsl").into()),
        });

        let downscale_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_blur downsample render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &downscale_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &downscale_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        let texture2 = Texture::new(device, size, format, &sampler);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_blur upsample render pipeline layout"),
            bind_group_layouts: &[&texture2.bind_group_layout, &texel_layout],
            push_constant_ranges: &[],
        });

        let upscale_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_blur upsample shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/upsample.wgsl").into()),
        });

        let upscale_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_blur upsample render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &upscale_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &upscale_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        Self {
            offset_alignment,
            upscale_pipeline,
            downscale_pipeline,
            texel_bind_group,
            sampler,
            textures: [texture1, texture2],
        }
    }

    fn update(&mut self, device: &wgpu::Device, size: Size<u32>) {
        for texture in &mut self.textures {
            texture.update(device, size, &self.sampler);
        }
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::Texture,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
        radius: u32,
    ) {
        let radius = radius.clamp(1, Shader::MAX_BLUR_RADIUS);

        // copy framebuffer into `textures[0]`
        {
            let source = wgpu::TexelCopyTextureInfoBase {
                origin: wgpu::Origin3d {
                    x: clip_bounds.x,
                    y: clip_bounds.y,
                    z: 0,
                },
                ..frame.as_image_copy()
            };

            let copy_size = wgpu::Extent3d {
                width: clip_bounds.width,
                height: clip_bounds.height,
                depth_or_array_layers: 1,
            };

            encoder.copy_texture_to_texture(
                source,
                self.textures[0].texture.as_image_copy(),
                copy_size,
            );
        }

        // downsample
        for i in 1..=radius {
            let (dst, src) = if i.is_multiple_of(2) {
                (&self.textures[0].view, &self.textures[1].bind_group)
            } else {
                (&self.textures[1].view, &self.textures[0].bind_group)
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_blur texture render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let offset =
                (i - 1 as wgpu::DynamicOffset) * (self.offset_alignment as wgpu::DynamicOffset);
            render_pass.set_pipeline(&self.downscale_pipeline);
            render_pass.set_bind_group(0, src, &[]);
            render_pass.set_bind_group(1, &self.texel_bind_group, &[offset]);
            render_pass.draw(0..6, 0..1);
        }

        // upsample
        for i in (0..radius).rev() {
            let (dst, src) = if i == 0 {
                (target, &self.textures[1].bind_group)
            } else if i.is_multiple_of(2) {
                (&self.textures[0].view, &self.textures[1].bind_group)
            } else {
                (&self.textures[1].view, &self.textures[0].bind_group)
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_blur texture render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let offset =
                (i as wgpu::DynamicOffset) * (self.offset_alignment as wgpu::DynamicOffset);

            if i == 0 {
                render_pass.set_viewport(
                    clip_bounds.x as f32,
                    clip_bounds.y as f32,
                    clip_bounds.width as f32,
                    clip_bounds.height as f32,
                    0.0,
                    1.0,
                );
            }
            render_pass.set_pipeline(&self.upscale_pipeline);
            render_pass.set_bind_group(0, src, &[]);
            render_pass.set_bind_group(1, &self.texel_bind_group, &[offset]);
            render_pass.draw(0..6, 0..1);
        }
    }
}

struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Texture {
    fn new(
        device: &wgpu::Device,
        size: Size<u32>,
        format: wgpu::TextureFormat,
        sampler: &wgpu::Sampler,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("iced_blur texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced_blur texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_blur bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        Self {
            texture,
            view,
            bind_group,
            bind_group_layout,
        }
    }

    fn update(&mut self, device: &wgpu::Device, size: Size<u32>, sampler: &wgpu::Sampler) {
        if self.texture.width() != size.width || self.texture.height() != size.height {
            *self = Texture::new(device, size, self.texture.format(), &sampler);
        }
    }
}
