use thiserror::Error;

/// Errors produced by the renderer.
#[derive(Debug, Error)]
pub enum RendererError {
    #[error("failed to create wgpu instance")]
    InstanceCreation,

    #[error("failed to create rendering surface")]
    SurfaceCreation,

    #[error("no suitable GPU adapter found")]
    AdapterNotFound,

    #[error("failed to request logical device: {0}")]
    DeviceRequest(String),

    #[error("surface has not been configured")]
    SurfaceNotConfigured,

    #[error("surface error: {0}")]
    Surface(String),

    #[error("shader error: {0}")]
    Shader(String),

    #[error("pipeline error: {0}")]
    Pipeline(String),

    #[error("buffer error: {0}")]
    Buffer(String),

    #[error("texture error: {0}")]
    Texture(String),

    #[error("mesh error: {0}")]
    Mesh(String),

    #[error("{0}")]
    Other(String),
}

impl From<wgpu::SurfaceError> for RendererError {
    fn from(value: wgpu::SurfaceError) -> Self {
        Self::Surface(value.to_string())
    }
}

impl From<wgpu::RequestDeviceError> for RendererError {
    fn from(value: wgpu::RequestDeviceError) -> Self {
        Self::DeviceRequest(value.to_string())
    }
}
