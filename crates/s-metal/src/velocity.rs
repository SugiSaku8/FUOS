//! GPU velocity field.

use crate::{GpuField3D, GridInfo, MetalContext};

use s_field::VelocityField;

/// GPU resident velocity field.
///
/// Components:
///
/// u : east-west velocity
/// v : north-south velocity
/// w : vertical velocity
///
pub struct GpuVelocityField {
    pub u: GpuField3D,

    pub v: GpuField3D,

    pub w: GpuField3D,

    grid: GridInfo,
}

impl GpuVelocityField {
    /// Upload CPU velocity field to GPU.
    pub fn from_velocity(context: &MetalContext, velocity: &VelocityField) -> Self {
        let shape = velocity.shape();

        let spacing = velocity.spacing();

        let grid = GridInfo::new(
            shape[0] as u32,
            shape[1] as u32,
            shape[2] as u32,
            spacing[0],
            spacing[1],
            spacing[2],
        );

        //
        // First time slice
        //
        let u = velocity.u().time_slice(0).unwrap();

        let v = velocity.v().time_slice(0).unwrap();

        let w = velocity.w().time_slice(0).unwrap();

        let gpu = Self {
            u: GpuField3D::from_f32(context, u, grid),

            v: GpuField3D::from_f32(context, v, grid),

            w: GpuField3D::from_f32(context, w, grid),

            grid,
        };

        //
        // Debug upload check
        //
        let center =
            (shape[2] / 2) * shape[0] * shape[1] + (shape[1] / 2) * shape[0] + shape[0] / 2;

        let uploaded = gpu.u.download(context);

        println!("GPU upload center = {}", uploaded[center]);

        gpu
    }

    /// Create empty velocity field.
    ///
    /// Used for compute output.
    pub fn empty(context: &MetalContext, grid: GridInfo) -> Self {
        Self {
            u: GpuField3D::empty(context, grid),

            v: GpuField3D::empty(context, grid),

            w: GpuField3D::empty(context, grid),

            grid,
        }
    }

    #[inline]
    pub fn grid(&self) -> GridInfo {
        self.grid
    }
}
