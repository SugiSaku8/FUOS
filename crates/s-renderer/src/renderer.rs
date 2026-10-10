use anyhow::Result;

use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use wgpu;

use crate::{
    camera::Camera, context::RenderContext, depth::DepthTexture, flow_mesh::GpuFlowMesh,
    flow_pipeline::FlowPipeline, mesh::GpuMesh, pipeline::RenderPipeline, shader::Shader,
    surface::RenderSurface, uniform::CameraBuffer, velocity_mesh::GpuVelocityMesh,
    velocity_pipeline::VelocityPipeline,
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

        let depth = DepthTexture::new(context.device(), size.width, size.height)?;

        let camera = Camera::new(size.width, size.height);

        let camera_buffer = CameraBuffer::new(context.device(), pipeline.camera_layout());

        Ok(Self {
            context,

            surface,

            pipeline,

            flow_pipeline,

            velocity_pipeline,

            depth,

            camera,

            camera_buffer,
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

        flow: Option<&GpuFlowMesh>,

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
