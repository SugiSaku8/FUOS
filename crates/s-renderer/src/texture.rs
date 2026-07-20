use anyhow::{Result, anyhow};

/// GPU texture resource.
///
/// Owns a texture and its default view.
#[derive(Debug)]
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    size: wgpu::Extent3d,
}

impl Texture {
    /// Creates a 2D texture.
    pub fn new_2d(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        label: &'static str,
    ) -> Result<Self> {
        if size.width == 0 || size.height == 0 {
            return Err(anyhow!("invalid texture size"));
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),

            size,

            mip_level_count: 1,

            sample_count: 1,

            dimension: wgpu::TextureDimension::D2,

            format,

            usage,

            view_formats: &[format],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            texture,
            view,
            format,
            size,
        })
    }

    /// Creates a render target texture.
    pub fn render_target(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self> {
        Self::new_2d(
            device,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            "render_target_texture",
        )
    }

    /// Creates a depth texture.
    pub fn depth(device: &wgpu::Device, width: u32, height: u32) -> Result<Self> {
        Self::new_2d(
            device,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            "depth_texture",
        )
    }

    #[inline]
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    #[inline]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    #[inline]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    #[inline]
    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }
}

/// GPU sampler wrapper.
#[derive(Debug)]
pub struct Sampler {
    sampler: wgpu::Sampler,
}

impl Sampler {
    pub fn linear(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear_sampler"),

            address_mode_u: wgpu::AddressMode::ClampToEdge,

            address_mode_v: wgpu::AddressMode::ClampToEdge,

            address_mode_w: wgpu::AddressMode::ClampToEdge,

            mag_filter: wgpu::FilterMode::Linear,

            min_filter: wgpu::FilterMode::Linear,

            mipmap_filter: wgpu::FilterMode::Linear,

            ..Default::default()
        });

        Self { sampler }
    }

    pub fn nearest(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest_sampler"),

            address_mode_u: wgpu::AddressMode::ClampToEdge,

            address_mode_v: wgpu::AddressMode::ClampToEdge,

            address_mode_w: wgpu::AddressMode::ClampToEdge,

            mag_filter: wgpu::FilterMode::Nearest,

            min_filter: wgpu::FilterMode::Nearest,

            mipmap_filter: wgpu::FilterMode::Nearest,

            ..Default::default()
        });

        Self { sampler }
    }

    #[inline]
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
