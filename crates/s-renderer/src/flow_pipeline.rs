use anyhow::Result;

pub struct FlowPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl FlowPipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        shader: &wgpu::ShaderModule,
        camera_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self> {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("flow_pipeline_layout"),

            bind_group_layouts: &[camera_layout],

            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("flow_pipeline"),

            layout: Some(&layout),

            vertex: wgpu::VertexState {
                module: shader,

                entry_point: Some("vs_main"),

                compilation_options: wgpu::PipelineCompilationOptions::default(),

                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as u64,

                    step_mode: wgpu::VertexStepMode::Vertex,

                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,

                        shader_location: 0,

                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
            },

            fragment: Some(wgpu::FragmentState {
                module: shader,

                entry_point: Some("fs_main"),

                compilation_options: wgpu::PipelineCompilationOptions::default(),

                targets: &[Some(wgpu::ColorTargetState {
                    format,

                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),

                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,

                ..Default::default()
            },

            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,

                depth_write_enabled: false,

                depth_compare: wgpu::CompareFunction::Less,

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
