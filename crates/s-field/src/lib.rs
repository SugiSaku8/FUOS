//! s-field
//!
//! SOS (Suruga Ocean Simulator) の海洋物理量ライブラリ。
//!
//! 主な機能
//! - 汎用2D/3D/4Dフィールド
//! - 海洋物理量（流速・水温・塩分）
//! - 座標系
//! - OceanFieldデータセット
pub mod boundary;
pub mod coordinates;
pub mod field;
pub mod field2d;
pub mod field3d;
pub mod field4d;
pub mod field_storage;
pub mod generator;
pub mod grid;
pub mod metadata;
pub mod ocean;
pub mod ocean_grid;
pub mod salinity;
pub mod temperature;
pub mod velocity;
pub use boundary::*;
pub use coordinates::*;
pub use field::*;
pub use field_storage::*;
pub mod scalar;
pub use field2d::Field2D;
pub use field3d::Field3D;
pub use field4d::Field4D;
pub use grid::{Grid2D, Grid3D, Grid4D};
pub use metadata::*;
pub use ocean::OceanField;
pub use ocean_grid::OceanGrid;
pub use salinity::SalinityField;
pub use scalar::ScalarField;
pub use temperature::TemperatureField;
pub use velocity::VelocityField;
