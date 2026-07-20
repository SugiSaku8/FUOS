use anyhow::Result;

use crate::texture::Texture;

/// Depth buffer used during rendering.
///
/// The depth texture is independent from the surface.
/// It is recreated whenever the window size changes.
#[derive(Debug)]
pub struct DepthTexture {
    texture: Texture,
}

impl DepthTexture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Result<Self> {
        let texture = Texture::depth(device, width, height)?;

        Ok(Self { texture })
    }

    /// Recreates the depth buffer after resize.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) -> Result<()> {
        self.texture = Texture::depth(device, width, height)?;

        Ok(())
    }

    #[inline]
    pub fn view(&self) -> &wgpu::TextureView {
        self.texture.view()
    }

    #[inline]
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Creates a RenderPass depth attachment.
    pub fn attachment(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: self.view(),

            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),

                store: wgpu::StoreOp::Store,
            }),

            stencil_ops: None,
        }
    }
}
