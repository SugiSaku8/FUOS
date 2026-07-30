use anyhow::Result;

use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    camera::Camera, context::RenderContext, depth::DepthTexture, mesh::GpuMesh,
    ocean::OceanRenderData, pipeline::RenderPipeline, shader::Shader, surface::RenderSurface,
    uniform::CameraBuffer,
};

pub struct Renderer {
    context: RenderContext,

    surface: RenderSurface,

    pipeline: RenderPipeline,

    depth: DepthTexture,

    pub camera: Camera,

    camera_buffer: CameraBuffer,

    ocean: Option<OceanRenderData>,
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

        let shader = Shader::from_wgsl(context.device(), shader_source, "sos_shader")?;

        let pipeline = RenderPipeline::new(
            context.device(),
            surface.format(),
            Some(wgpu::TextureFormat::Depth32Float),
            vertex_layouts,
            shader.module(),
        )?;

        let depth = DepthTexture::new(context.device(), size.width, size.height)?;

        let camera = Camera::new(size.width, size.height);

        let camera_buffer = CameraBuffer::new(context.device(), pipeline.camera_layout());

        Ok(Self {
            context,

            surface,

            pipeline,

            depth,

            camera,

            camera_buffer,

            ocean: None,
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
            .expect("depth resize failed");

        self.camera.resize(size.width, size.height);
    }

    pub fn render<V>(&mut self, mesh: &GpuMesh<V>) -> Result<()>
    where
        V: bytemuck::Pod,
    {
        self.camera_buffer
            .uniform
            .update(self.camera.build_matrix());

        self.camera_buffer.update(self.context.queue());

        let frame = self.surface.get_current_texture()?;

        let view = frame.texture.create_view(&Default::default());

        let mut encoder =
            self.context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("terrain_encoder"),
                });

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

            pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));

            pass.set_index_buffer(mesh.index_buffer().slice(..), wgpu::IndexFormat::Uint32);

            pass.draw_indexed(0..mesh.index_count(), 0, 0..1);
        }

        self.context.queue().submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }

    pub fn set_ocean(&mut self, ocean: OceanRenderData) {
        self.ocean = Some(ocean);
    }

    pub fn clear_ocean(&mut self) {
        self.ocean = None;
    }

    pub fn ocean(&self) -> Option<&OceanRenderData> {
        self.ocean.as_ref()
    }

    pub fn update_camera(&mut self) {
        self.camera_buffer
            .uniform
            .update(self.camera.build_matrix());

        self.camera_buffer.update(self.context.queue());
    }
}
