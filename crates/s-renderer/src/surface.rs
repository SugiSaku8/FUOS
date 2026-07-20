use anyhow::{anyhow, Result};
use winit::{dpi::PhysicalSize, window::Window};

use crate::context::RenderContext;

/// Window rendering surface.
///
/// Manages swapchain configuration only.
/// GPU resources are owned by RenderContext.
#[derive(Debug)]
pub struct RenderSurface<'window> {
    surface: wgpu::Surface<'window>,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    format: wgpu::TextureFormat,
}

impl<'window> RenderSurface<'window> {
    pub fn new(context: &RenderContext, window: &'window Window) -> Result<Self> {
        let size = window.inner_size();

        let surface = context
            .instance()
            .create_surface(window)
            .map_err(|e| anyhow!("failed to create wgpu surface: {e}"))?;

        let capabilities = surface.get_capabilities(context.adapter());

        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| {
                matches!(
                    format,
                    wgpu::TextureFormat::Bgra8UnormSrgb | wgpu::TextureFormat::Rgba8UnormSrgb
                )
            })
            .unwrap_or(capabilities.formats[0]);

        let present_mode = if capabilities
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        };

        let alpha_mode = capabilities.alpha_modes[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,

            format,

            width: size.width.max(1),

            height: size.height.max(1),

            present_mode,

            alpha_mode,

            view_formats: vec![format],

            desired_maximum_frame_latency: 2,
        };

        surface.configure(context.device(), &config);

        Ok(Self {
            surface,
            config,
            size,
            format,
        })
    }

    pub fn resize(&mut self, context: &RenderContext, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.size = size;

        self.config.width = size.width;

        self.config.height = size.height;

        self.surface.configure(context.device(), &self.config);
    }

    pub fn recover(&mut self, context: &RenderContext) {
        self.surface.configure(context.device(), &self.config);
    }

    pub fn acquire(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    #[inline]
    pub fn surface(&self) -> &wgpu::Surface<'window> {
        &self.surface
    }

    #[inline]
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    #[inline]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    #[inline]
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
}
