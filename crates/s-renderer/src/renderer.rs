use anyhow::Result;

use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use wgpu;

use crate::terrain_mesh::TerrainVertex;
use crate::{
    camera::Camera, context::RenderContext, depth::DepthTexture, flow_mesh::GpuFlowMesh,
    flow_pipeline::FlowPipeline, mesh::GpuMesh, pipeline::RenderPipeline, shader::Shader,
    surface::RenderSurface, terrain_mesh::TerrainVertex, uniform::CameraBuffer,
    velocity_mesh::GpuVelocityMesh, velocity_pipeline::VelocityPipeline,
};

pub struct Renderer {
    context: RenderContext,

    surface: RenderSurface,

    //
    // Terrain
    //
    pipeline: RenderPipeline,

    //
    // Flow
    //
    flow_pipeline: FlowPipeline,

    ocean_pipeline: wgpu::RenderPipeline,
    flow_dot_pipeline: wgpu::RenderPipeline,

    //
    // Velocity
    //
    pub velocity_pipeline: VelocityPipeline,

    //
    // Depth
    //
    depth: DepthTexture,

    pub camera: Camera,

    camera_buffer: CameraBuffer,
    ocean_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        shader_source: &str,
        vertex_layouts: &[wgpu::VertexBufferLayout<'static>],
    ) -> Result<Self> {
        let context = RenderContext::new().await?;

        let surface = RenderSurface::new(&context, window)?;

        let size = surface.size();

        //
        // Terrain Pipeline
        //

        let shader = Shader::from_wgsl(context.device(), shader_source, "terrain_shader")?;

        let pipeline = RenderPipeline::new(
            context.device(),
            surface.format(),
            Some(wgpu::TextureFormat::Depth32Float),
            vertex_layouts,
            shader.module(),
        )?;

        //
        // Flow Pipeline
        //

        let flow_shader =
            Shader::from_wgsl(context.device(), include_str!("flow.wgsl"), "flow_shader")?;

        let flow_pipeline = FlowPipeline::new(
            context.device(),
            surface.format(),
            flow_shader.module(),
            pipeline.camera_layout(),
        )?;

        let ocean_shader = Shader::from_wgsl(
            context.device(),
            include_str!("ocean_surface.wgsl"),
            "ocean_surface_shader",
        )?;
        let ocean_pipeline = create_auxiliary_pipeline(
            context.device(),
            surface.format(),
            Some(wgpu::TextureFormat::Depth32Float),
            ocean_shader.module(),
            pipeline.camera_layout(),
            &TerrainVertex::layout(),
            wgpu::PrimitiveTopology::TriangleList,
            true,
        );

        let dot_pipeline = create_auxiliary_pipeline(
            context.device(),
            surface.format(),
            Some(wgpu::TextureFormat::Depth32Float),
            flow_shader.module(),
            pipeline.camera_layout(),
            &crate::flow::FlowVertex::layout(),
            wgpu::PrimitiveTopology::PointList,
            false,
        );

        //
        // Velocity Pipeline
        //

        let velocity_shader = Shader::from_wgsl(
            context.device(),
            include_str!("velocity.wgsl"),
            "velocity_shader",
        )?;

        let velocity_pipeline = VelocityPipeline::new(
            context.device(),
            surface.format(),
            velocity_shader.module(),
            pipeline.camera_layout(),
        )?;

        let ocean_shader = Shader::from_wgsl(
            context.device(),
            include_str!("ocean_surface.wgsl"),
            "ocean_surface_shader",
        )?;

        let ocean_pipeline = create_ocean_pipeline(
            context.device(),
            surface.format(),
            ocean_shader.module(),
            pipeline.camera_layout(),
        );

        let depth = DepthTexture::new(context.device(), size.width, size.height)?;

        let camera = Camera::new(size.width, size.height);

        let camera_buffer = CameraBuffer::new(context.device(), pipeline.camera_layout());

        Ok(Self {
            context,

            surface,

            pipeline,

            flow_pipeline,

            ocean_pipeline,
            flow_dot_pipeline: dot_pipeline,

            velocity_pipeline,

            depth,

            camera,

            camera_buffer,
            ocean_pipeline,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        self.context.device()
    }

    pub fn queue(&self) -> &wgpu::Queue {
        self.context.queue()
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.surface.resize(&self.context, size);

        self.depth
            .resize(self.context.device(), size.width, size.height)
            .unwrap();

        self.camera.resize(size.width, size.height);
    }

    pub fn render<V>(
        &mut self,
        terrain: &GpuMesh<V>,
        ocean: Option<&GpuMesh<TerrainVertex>>,
        flow: Option<&GpuFlowMesh>,
        velocity: Option<&GpuVelocityMesh>,
    ) -> Result<()>
    where
        V: bytemuck::Pod,
    {
        self.render_with_ocean_and_particles(terrain, None, flow, None, velocity)
    }

    /// 海底地形、半透明海面、流線、粒子位置をまとめて描画する。
    pub fn render_with_ocean_and_particles<V>(
        &mut self,
        terrain: &GpuMesh<V>,
        ocean: Option<&GpuMesh<V>>,
        flow: Option<&GpuFlowMesh>,
        particle_dots: Option<&GpuFlowMesh>,
        velocity: Option<&GpuVelocityMesh>,
    ) -> Result<()>
    where
        V: bytemuck::Pod,
    {
        self.camera_buffer
            .uniform
            .update(self.camera.build_matrix());

        self.camera_buffer.update(self.context.queue());

        let frame = self.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("sos_render_encoder"),
                });

        //
        // Terrain Pass
        //

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("terrain_pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,

                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.08,
                            b: 0.15,
                            a: 1.0,
                        }),

                        store: wgpu::StoreOp::Store,
                    },

                    depth_slice: None,
                })],

                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth.view(),

                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),

                        store: wgpu::StoreOp::Store,
                    }),

                    stencil_ops: None,
                }),

                timestamp_writes: None,

                occlusion_query_set: None,
            });

            pass.set_pipeline(self.pipeline.pipeline());

            pass.set_bind_group(0, &self.camera_buffer.bind_group, &[]);

            pass.set_vertex_buffer(0, terrain.vertex_buffer().slice(..));

            pass.set_index_buffer(terrain.index_buffer().slice(..), wgpu::IndexFormat::Uint32);

            pass.draw_indexed(0..terrain.index_count(), 0, 0..1);
        }

        // 海面は海底地形の後にアルファブレンドする。深度書き込みは行わない。
        if let Some(ocean) = ocean {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("transparent_ocean_surface_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.ocean_pipeline);
            pass.set_bind_group(0, &self.camera_buffer.bind_group, &[]);
            pass.set_vertex_buffer(0, ocean.vertex_buffer().slice(..));
            pass.set_index_buffer(ocean.index_buffer().slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..ocean.index_count(), 0, 0..1);
        }

        //
        // Flow Pass
        //
        if let Some(flow) = flow {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("flow_pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,

                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,

                        store: wgpu::StoreOp::Store,
                    },

                    depth_slice: None,
                })],

                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth.view(),

                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,

                        store: wgpu::StoreOp::Store,
                    }),

                    stencil_ops: None,
                }),

                timestamp_writes: None,

                occlusion_query_set: None,
            });

            pass.set_pipeline(self.flow_pipeline.pipeline());

            pass.set_bind_group(0, &self.camera_buffer.bind_group, &[]);

            pass.set_vertex_buffer(0, flow.vertex_buffer().slice(..));

            pass.draw(0..flow.vertex_count(), 0..1);
        }

        // 各粒子の現在位置を点として描画する。
        if let Some(dots) = particle_dots {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("flow_particle_dots_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.flow_dot_pipeline);
            pass.set_bind_group(0, &self.camera_buffer.bind_group, &[]);
            pass.set_vertex_buffer(0, dots.vertex_buffer().slice(..));
            pass.draw(0..dots.vertex_count(), 0..1);
        }

        //
        // Velocity Pass
        //
        if let Some(velocity) = velocity {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("velocity_pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,

                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,

                        store: wgpu::StoreOp::Store,
                    },

                    depth_slice: None,
                })],

                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth.view(),

                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,

                        store: wgpu::StoreOp::Store,
                    }),

                    stencil_ops: None,
                }),

                timestamp_writes: None,

                occlusion_query_set: None,
            });

            pass.set_pipeline(self.velocity_pipeline.pipeline());

            pass.set_bind_group(0, &self.camera_buffer.bind_group, &[]);

            pass.set_vertex_buffer(0, velocity.vertex_buffer().slice(..));

            pass.draw(0..velocity.vertex_count(), 0..1);
        }

        //
        // Submit
        //
        self.context.queue().submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }

    pub fn update_camera(&mut self) {
        self.camera_buffer
            .uniform
            .update(self.camera.build_matrix());

        self.camera_buffer.update(self.context.queue());
    }

    //
    // Velocity only renderer
    //
    pub fn render_velocity(&mut self, velocity: &GpuVelocityMesh) -> Result<()> {
        self.camera_buffer
            .uniform
            .update(self.camera.build_matrix());

        self.camera_buffer.update(self.context.queue());

        let frame = self.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("velocity_encoder"),
                });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("velocity_only_pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,

                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.05,
                            b: 0.1,
                            a: 1.0,
                        }),

                        store: wgpu::StoreOp::Store,
                    },

                    depth_slice: None,
                })],

                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth.view(),

                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),

                        store: wgpu::StoreOp::Store,
                    }),

                    stencil_ops: None,
                }),

                timestamp_writes: None,

                occlusion_query_set: None,
            });

            pass.set_pipeline(self.velocity_pipeline.pipeline());

            pass.set_bind_group(0, &self.camera_buffer.bind_group, &[]);

            pass.set_vertex_buffer(0, velocity.vertex_buffer().slice(..));

            pass.draw(0..velocity.vertex_count(), 0..1);
        }

        self.context.queue().submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }
}

fn create_auxiliary_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    shader: &wgpu::ShaderModule,
    camera_layout: &wgpu::BindGroupLayout,
    vertex_layout: &wgpu::VertexBufferLayout<'static>,
    topology: wgpu::PrimitiveTopology,
    alpha_blending: bool,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("sos_auxiliary_pipeline_layout"),
        bind_group_layouts: &[camera_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("sos_auxiliary_pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: std::slice::from_ref(vertex_layout),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(if alpha_blending {
                    wgpu::BlendState::ALPHA_BLENDING
                } else {
                    wgpu::BlendState::REPLACE
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn create_ocean_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    shader: &wgpu::ShaderModule,
    camera_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("ocean_pipeline_layout"),
        bind_group_layouts: &[camera_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ocean_pipeline"),
        layout: Some(&layout),

        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[TerrainVertex::layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },

        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),

            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,

                // アルファブレンディングを有効にする。
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),

                write_mask: wgpu::ColorWrites::ALL,
            })],

            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),

        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..Default::default()
        },

        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,

            // 海面で深度バッファを上書きしない。
            depth_write_enabled: false,

            depth_compare: wgpu::CompareFunction::LessEqual,

            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),

        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
