use thiserror::Error;

#[derive(Debug, Error)]
pub enum IoError {
    #[error("NetCDF Error: {0}")]
    NetCDF(#[from] netcdf::Error),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Dimension mismatch")]
    DimensionMismatch,
}
