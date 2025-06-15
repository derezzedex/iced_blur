use iced_core::Rectangle;
use iced_core::Size;
use iced_core::mouse;
use iced_widget::renderer::wgpu::wgpu;
use iced_widget::shader;

pub struct Shader;

impl<Message> shader::Program<Message> for Shader {
    type State = ();
    type Primitive = Primitive;

    fn draw(&self, _state: &Self::State, _cursor: mouse::Cursor, _bounds: Rectangle) -> Primitive {
        Primitive
    }
}

#[derive(Debug)]
pub struct Primitive;

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        frame: &wgpu::Texture,
        storage: &mut shader::Storage,
        bounds: &Rectangle,
        viewport: &shader::Viewport,
    ) {
        let size = Size::new(
            (bounds.width * viewport.scale_factor() as f32).round() as u32,
            (bounds.height * viewport.scale_factor() as f32).round() as u32,
        );
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, size, frame.format()));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();
        pipeline.update(device, size);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        frame: &wgpu::Texture,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        storage
            .get::<Pipeline>()
            .unwrap()
            .render(encoder, frame, target, clip_bounds);
    }
}

pub struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    texture: Texture,
    sampler: wgpu::Sampler,
}

impl Pipeline {
    fn new(device: &wgpu::Device, size: Size<u32>, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_blur sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let texture = Texture::new(device, size, format, &sampler);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_blur render pipeline layout"),
            bind_group_layouts: &[&texture.bind_group_layout],
            push_constant_ranges: &[],
        });

        let paste_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_blur paste shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/paste.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_blur render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &paste_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &paste_shader,
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
            texture,
            sampler,
            render_pipeline,
        }
    }

    fn update(&mut self, device: &wgpu::Device, size: Size<u32>) {
        self.texture.update(device, size, &self.sampler);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::Texture,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // copy framebuffer into texture
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
                self.texture.texture.as_image_copy(),
                copy_size,
            );
        }

        // render texture into framebuffer
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_blur texture render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
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
            render_pass.set_bind_group(0, &self.texture.bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..6, 0..1);
        }
    }
}

struct Texture {
    texture: wgpu::Texture,
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
