//! Export GPU ocean fields.

use crate::{GpuVelocityField, MetalContext};

/// CPU side velocity snapshot.
///
/// Used for renderer / visualization.
pub struct VelocitySnapshot {
    pub u: Vec<f32>,

    pub v: Vec<f32>,

    pub w: Vec<f32>,

    pub nx: usize,

    pub ny: usize,

    pub nz: usize,
}

impl VelocitySnapshot {
    pub fn size(&self) -> usize {
        self.nx * self.ny * self.nz
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        z * self.nx * self.ny + y * self.nx + x
    }
}

impl GpuVelocityField {
    /// Download GPU velocity field.
    ///
    /// GPU
    ///  |
    ///  v
    /// CPU snapshot
    ///
    pub fn snapshot(&self, context: &MetalContext) -> VelocitySnapshot {
        let u = self.u.download(context);

        let v = self.v.download(context);

        let w = self.w.download(context);

        let grid = self.grid();

        VelocitySnapshot {
            u,

            v,

            w,

            nx: grid.nx() as usize,

            ny: grid.ny() as usize,

            nz: grid.nz() as usize,
        }
    }
}
