use std::sync::Arc;

use anyhow::Result;

use winit::{dpi::PhysicalSize, window::Window};

use crate::context::RenderContext;

pub struct RenderSurface {
    surface: wgpu::Surface<'static>,

    config: wgpu::SurfaceConfiguration,

    size: PhysicalSize<u32>,
}

impl RenderSurface {
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        Ok(self.surface.get_current_texture()?)
    }
    pub fn new(context: &RenderContext, window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

        let surface = context.instance().create_surface(window)?;

        let capabilities = surface.get_capabilities(context.adapter());

        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,

            format,

            width: size.width,

            height: size.height,

            // Disable 60 Hz FIFO vsync; the simulator event loop applies a 100 FPS cap.
            present_mode: wgpu::PresentMode::AutoNoVsync,

            alpha_mode: capabilities.alpha_modes[0],

            view_formats: vec![],

            desired_maximum_frame_latency: 2,
        };

        surface.configure(context.device(), &config);

        Ok(Self {
            surface,

            config,

            size,
        })
    }

    pub fn acquire(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        Ok(self.surface.get_current_texture()?)
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

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
}
