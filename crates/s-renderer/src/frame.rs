use crate::{context::RenderContext, surface::RenderSurface};

pub struct Frame {
    surface_texture: wgpu::SurfaceTexture,

    encoder: wgpu::CommandEncoder,
}

impl Frame {
    pub fn begin(
        context: &RenderContext,

        surface: &RenderSurface,
    ) -> Result<(Self, wgpu::TextureView), wgpu::SurfaceError> {
        let surface_texture = surface.acquire()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = context
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("sos_encoder"),
            });

        Ok((
            Self {
                surface_texture,

                encoder,
            },
            view,
        ))
    }

    pub fn render_pass<'a>(
        &'a mut self,

        descriptor: &'a wgpu::RenderPassDescriptor<'a>,
    ) -> wgpu::RenderPass<'a> {
        self.encoder.begin_render_pass(descriptor)
    }

    pub fn finish(self, context: &RenderContext) {
        let command_buffer = self.encoder.finish();

        context.queue().submit(std::iter::once(command_buffer));

        self.surface_texture.present();
    }
}
