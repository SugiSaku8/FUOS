use anyhow::Result;

use winit::{dpi::PhysicalSize, window::Window};

use bytemuck::Pod;

use crate::{
    camera::Camera, context::RenderContext, depth::DepthTexture, frame::Frame, mesh::GpuMesh,
    pipeline::RenderPipeline, shader::Shader, surface::RenderSurface, uniform::CameraBuffer,
};

#[derive(Debug)]
pub struct Renderer<'window> {
    context: RenderContext,

    surface: RenderSurface<'window>,

    depth: DepthTexture,

    camera: Camera,

    camera_buffer: CameraBuffer,

    pipeline: RenderPipeline,
}

impl<'window> Renderer<'window> {
    pub async fn new(
        window: &'window Window,

        shader_source: &str,

        vertex_layouts: &'static [wgpu::VertexBufferLayout<'static>],
    ) -> Result<Self> {
        let context = RenderContext::new().await?;

        let surface = RenderSurface::new(&context, window)?;

        let size = surface.size();

        let depth = DepthTexture::new(context.device(), size.width, size.height)?;

        let camera = Camera::default_perspective(size.width, size.height);

        let camera_buffer = CameraBuffer::new(context.device());

        let shader = Shader::from_wgsl(context.device(), shader_source, "terrain_shader")?;

        let pipeline = RenderPipeline::new(
            context.device(),
            surface.format(),
            Some(wgpu::TextureFormat::Depth32Float),
            camera_buffer.bind_group_layout(),
            vertex_layouts,
            shader.module(),
        )?;

        Ok(Self {
            context,

            surface,

            depth,

            camera,

            camera_buffer,

            pipeline,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface.resize(&self.context, size);

        let _ = self
            .depth
            .resize(self.context.device(), size.width, size.height);

        self.camera.resize(size.width, size.height);
    }

    pub fn render<V>(&mut self, mesh: &GpuMesh<V>) -> Result<(), wgpu::SurfaceError>
    where
        V: Pod,
    {
        let mut frame = Frame::begin(&self.context, &self.surface)?;

        self.camera_buffer.update(
            self.context.queue(),
            &self.camera.view_matrix(),
            &self.camera.projection_matrix(),
        );

        {
            let mut pass = frame.render_pass(&wgpu::RenderPassDescriptor {
                label: Some("terrain_render_pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame.view(),
                    depth_slice: None,
                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,

                            g: 0.05,

                            b: 0.08,

                            a: 1.0,
                        }),

                        store: wgpu::StoreOp::Store,
                    },
                })],

                depth_stencil_attachment: Some(self.depth.attachment()),

                timestamp_writes: None,

                occlusion_query_set: None,
            });

            pass.set_pipeline(self.pipeline.pipeline());

            pass.set_bind_group(0, self.camera_buffer.bind_group(), &[]);

            mesh.bind(&mut pass);

            mesh.draw(&mut pass);
        }

        frame.finish(&self.context);

        Ok(())
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
    #[inline]
    pub fn context(&self) -> &RenderContext {
        &self.context
    }

    #[inline]
    pub fn surface_size(&self) -> PhysicalSize<u32> {
        self.surface.size()
    }
}
