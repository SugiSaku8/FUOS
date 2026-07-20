use crate::{context::RenderContext, surface::RenderSurface};

/// Represents one rendering frame.
///
/// Lifecycle:
///
/// acquire
///   ↓
/// create render pass
///   ↓
/// submit commands
///   ↓
/// present
///
#[derive(Debug)]
pub struct Frame {
    surface_texture: wgpu::SurfaceTexture,

    view: wgpu::TextureView,

    encoder: wgpu::CommandEncoder,
}

impl Frame {
    /// Starts a new frame.
    pub fn begin(
        context: &RenderContext,
        surface: &RenderSurface,
    ) -> Result<Self, wgpu::SurfaceError> {
        let surface_texture = surface.acquire()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = context
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("sos_frame_encoder"),
            });

        Ok(Self {
            surface_texture,
            view,
            encoder,
        })
    }

    /// Returns the current frame texture view.
    #[inline]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Creates a render pass.
    pub fn render_pass<'a>(
        &'a mut self,
        descriptor: &wgpu::RenderPassDescriptor<'a>,
    ) -> wgpu::RenderPass<'a> {
        self.encoder.begin_render_pass(descriptor)
    }

    /// Finish commands and display frame.
    pub fn finish(self, context: &RenderContext) {
        let command_buffer = self.encoder.finish();

        context.queue().submit(std::iter::once(command_buffer));

        self.surface_texture.present();
    }
}
