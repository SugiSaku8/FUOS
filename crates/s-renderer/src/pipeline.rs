use anyhow::Result;

/// GPU render pipeline.
///
/// This type does not know about mesh formats.
/// Vertex layout is provided externally.
#[derive(Debug)]
pub struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    pub fn new(
        device: &wgpu::Device,

        color_format: wgpu::TextureFormat,

        depth_format: Option<wgpu::TextureFormat>,

        camera_layout: &wgpu::BindGroupLayout,

        vertex_layouts: &'static [wgpu::VertexBufferLayout<'static>],

        shader: &wgpu::ShaderModule,
    ) -> Result<Self> {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sos_pipeline_layout"),

            bind_group_layouts: &[camera_layout],

            push_constant_ranges: &[],
        });

        let depth = depth_format.map(|format| wgpu::DepthStencilState {
            format,

            depth_write_enabled: true,

            depth_compare: wgpu::CompareFunction::Less,

            stencil: wgpu::StencilState::default(),

            bias: wgpu::DepthBiasState::default(),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sos_render_pipeline"),

            layout: Some(&layout),

            vertex: wgpu::VertexState {
                module: shader,

                entry_point: Some("vs_main"),

                buffers: vertex_layouts,

                compilation_options: Default::default(),
            },

            fragment: Some(wgpu::FragmentState {
                module: shader,

                entry_point: Some("fs_main"),

                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,

                    blend: Some(wgpu::BlendState::REPLACE),

                    write_mask: wgpu::ColorWrites::ALL,
                })],

                compilation_options: Default::default(),
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,

                strip_index_format: None,

                front_face: wgpu::FrontFace::Ccw,

                cull_mode: Some(wgpu::Face::Back),

                polygon_mode: wgpu::PolygonMode::Fill,

                unclipped_depth: false,

                conservative: false,
            },

            depth_stencil: depth,

            multisample: wgpu::MultisampleState::default(),

            multiview: None,

            cache: None,
        });

        Ok(Self { pipeline })
    }

    #[inline]
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
