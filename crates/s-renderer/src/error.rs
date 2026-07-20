use thiserror::Error;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("texture creation failed")]
    TextureCreation,

    #[error("surface error: {0}")]
    Surface(String),

    #[error("gpu error: {0}")]
    Gpu(String),
}
