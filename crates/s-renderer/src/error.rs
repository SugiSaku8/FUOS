use thiserror::Error;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("GPU initialization failed")]
    Initialize,
}
