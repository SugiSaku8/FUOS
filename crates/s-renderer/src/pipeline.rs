use anyhow::Result;

pub struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,

    camera_layout: wgpu::BindGroupLayout,
}

impl RenderPipeline {
    pub fn new(
        device: &wgpu::Device,

        format: wgpu::TextureFormat,

        depth_format: Option<wgpu::TextureFormat>,

        vertex_layouts: &[wgpu::VertexBufferLayout<'static>],

        shader: &wgpu::ShaderModule,
    ) -> Result<Self> {
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),

            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,

                visibility: wgpu::ShaderStages::VERTEX,

                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,

                    has_dynamic_offset: false,

                    min_binding_size: Some(
                        std::num::NonZeroU64::new(
                            std::mem::size_of::<crate::uniform::CameraUniform>() as u64,
                        )
                        .unwrap(),
                    ),
                },

                count: None,
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sos_pipeline_layout"),

            bind_group_layouts: &[&camera_layout],

            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sos_render_pipeline"),

            layout: Some(&layout),

            vertex: wgpu::VertexState {
                module: shader,

                entry_point: Some("vs_main"),

                buffers: vertex_layouts,

                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },

            fragment: Some(wgpu::FragmentState {
                module: shader,

                entry_point: Some("fs_main"),

                targets: &[Some(wgpu::ColorTargetState {
                    format,

                    blend: Some(wgpu::BlendState::REPLACE),

                    write_mask: wgpu::ColorWrites::ALL,
                })],

                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,

                strip_index_format: None,

                front_face: wgpu::FrontFace::Ccw,

                // まずは無効化して確認
                cull_mode: None,

                polygon_mode: wgpu::PolygonMode::Fill,

                unclipped_depth: false,

                conservative: false,
            },

            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,

                depth_write_enabled: true,

                depth_compare: wgpu::CompareFunction::Less,

                stencil: wgpu::StencilState::default(),

                bias: wgpu::DepthBiasState::default(),
            }),

            multisample: wgpu::MultisampleState::default(),

            multiview: None,

            cache: None,
        });

        Ok(Self {
            pipeline,

            camera_layout,
        })
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn camera_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_layout
    }
}
