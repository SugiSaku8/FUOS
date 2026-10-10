pub struct VelocityPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl VelocityPipeline {
    pub fn new(
        device: &wgpu::Device,

        format: wgpu::TextureFormat,

        shader: &wgpu::ShaderModule,

        camera_layout: &wgpu::BindGroupLayout,
    ) -> anyhow::Result<Self> {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("velocity_pipeline_layout"),

            bind_group_layouts: &[camera_layout],

            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("velocity_pipeline"),

            layout: Some(&layout),

            vertex: wgpu::VertexState {
                module: shader,

                entry_point: Some("vs_main"),

                buffers: &[crate::velocity::VelocityVertex::layout()],
                compilation_options: Default::default(),
            },

            fragment: Some(wgpu::FragmentState {
                module: shader,

                entry_point: Some("fs_main"),

                targets: &[Some(wgpu::ColorTargetState {
                    format,

                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),

                    write_mask: wgpu::ColorWrites::ALL,
                })],

                compilation_options: Default::default(),
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,

                strip_index_format: None,

                front_face: wgpu::FrontFace::Ccw,

                cull_mode: None,

                polygon_mode: wgpu::PolygonMode::Fill,

                unclipped_depth: false,

                conservative: false,
            },

            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,

                depth_write_enabled: false,

                depth_compare: wgpu::CompareFunction::LessEqual,

                stencil: Default::default(),

                bias: Default::default(),
            }),

            multisample: wgpu::MultisampleState::default(),

            multiview: None,

            cache: None,
        });

        Ok(Self { pipeline })
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
