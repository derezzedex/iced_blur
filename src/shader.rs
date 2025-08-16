use iced_core::Rectangle;
use iced_core::Size;
use iced_core::mouse;
use iced_widget::renderer::wgpu::wgpu;
use iced_widget::renderer::wgpu::wgpu::util::DeviceExt;
use iced_widget::shader;

#[derive(Debug, Clone, Copy)]
pub struct Shader {
    passes: u32,
    offset: f32,
}

impl Shader {
    pub fn new(passes: u32, offset: f32) -> Self {
        Self { passes, offset }
    }
}

impl<Message> shader::Program<Message> for Shader {
    type State = ();
    type Primitive = Self;

    fn draw(&self, _state: &Self::State, _cursor: mouse::Cursor, _bounds: Rectangle) -> Self {
        Self {
            passes: self.passes,
            offset: self.offset,
        }
    }
}

impl shader::Primitive for Shader {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
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
            storage.store(Pipeline::new(device, size, format, *self));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();
        pipeline.update(queue, device, size, self);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        storage
            .get::<Pipeline>()
            .unwrap()
            .render(encoder, target.texture(), target, clip_bounds);
    }
}

pub struct Pipeline {
    blur: Shader,
    downscale_pipeline: wgpu::RenderPipeline,
    upscale_pipeline: wgpu::RenderPipeline,
    offset_bind_group: wgpu::BindGroup,
    offset_buffer: wgpu::Buffer,
    base: Texture,
    textures: [Texture; 5],
    sampler: wgpu::Sampler,
}

impl Pipeline {
    fn new(
        device: &wgpu::Device,
        size: Size<u32>,
        format: wgpu::TextureFormat,
        blur: Shader,
    ) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_blur sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let offset_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced_blur texel bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<f32>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            }],
        });

        let offset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("texel size buffer"),
            contents: bytemuck::cast_slice(&[blur.offset]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let offset_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_blur texel bind group"),
            layout: &offset_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    size: wgpu::BufferSize::new(std::mem::size_of::<f32>() as u64),
                    ..offset_buffer.as_entire_buffer_binding()
                }),
            }],
        });

        let size = Size::new(size.width, size.height);
        let texture1 = Texture::new(device, size, format, &sampler);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_blur downsample render pipeline layout"),
            bind_group_layouts: &[&texture1.bind_group_layout, &offset_layout],
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

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_blur upsample render pipeline layout"),
            bind_group_layouts: &[&texture1.bind_group_layout, &offset_layout],
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

        let textures = std::array::from_fn(|i| 2u32.pow(i as u32 + 1).min(16))
            .map(|level| Size::new(size.width / level, size.height / level))
            .map(|size| Texture::new(device, size, format, &sampler));

        Self {
            blur,
            upscale_pipeline,
            downscale_pipeline,
            offset_bind_group,
            offset_buffer,
            sampler,
            base: texture1,
            textures,
        }
    }

    fn update(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        size: Size<u32>,
        blur: &Shader,
    ) {
        if blur.offset != self.blur.offset {
            queue.write_buffer(&self.offset_buffer, 0, bytemuck::cast_slice(&[blur.offset]));
        }

        self.blur = *blur;

        self.base.update(device, size, &self.sampler);
        for (i, texture) in self.textures.iter_mut().enumerate() {
            let level = 2u32.pow(i as u32 + 1).min(16);
            texture.update(
                device,
                Size::new(size.width / level, size.height / level),
                &self.sampler,
            );
        }
    }

    fn downsample(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::BindGroup,
        dst: &wgpu::TextureView,
    ) {
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

        render_pass.set_pipeline(&self.downscale_pipeline);
        render_pass.set_bind_group(0, src, &[]);
        render_pass.set_bind_group(1, &self.offset_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    fn upsample(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::BindGroup,
        dst: &wgpu::TextureView,
    ) {
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

        render_pass.set_pipeline(&self.upscale_pipeline);
        render_pass.set_bind_group(0, src, &[]);
        render_pass.set_bind_group(1, &self.offset_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::Texture,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
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

            encoder.copy_texture_to_texture(source, self.base.texture.as_image_copy(), copy_size);
        }

        // first pass
        self.downsample(encoder, &self.base.bind_group, &self.textures[0].view);

        // downsample
        for i in 1..self.blur.passes as usize {
            let idx = if i <= 4 { i } else { 4 - (i % 2) };
            let (src, dst) = (&self.textures[idx - 1].bind_group, &self.textures[idx].view);

            self.downsample(encoder, src, dst);
        }

        // upsample
        for i in (1..self.blur.passes.min(4) as usize).rev() {
            let (src, dst) = (&self.textures[i].bind_group, &self.textures[i - 1].view);

            self.upsample(encoder, src, dst);
        }

        // blit
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iced_blur texture render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
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

        render_pass.set_viewport(
            clip_bounds.x as f32,
            clip_bounds.y as f32,
            clip_bounds.width as f32,
            clip_bounds.height as f32,
            0.0,
            1.0,
        );
        render_pass.set_pipeline(&self.upscale_pipeline);
        render_pass.set_bind_group(0, &self.textures[0].bind_group, &[]);
        render_pass.set_bind_group(1, &self.offset_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
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
