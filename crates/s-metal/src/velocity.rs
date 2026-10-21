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

    pub(crate) grid: GridInfo,
}

impl GpuVelocityField {
    /// Upload CPU velocity field to GPU.
    pub fn from_velocity(context: &MetalContext, velocity: &VelocityField) -> Self {
        let shape = velocity.shape();

        let spacing = velocity.spacing();

        // CPU storage is [depth, latitude, longitude] with longitude contiguous.
        // Metal kernels index x=longitude, y=latitude, z=depth.
        let grid = GridInfo::new(
            shape[2] as u32,
            shape[1] as u32,
            shape[0] as u32,
            spacing[3],
            spacing[2],
            spacing[1],
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

    /// Download GPU velocity field to CPU.
    ///
    /// Converts GPU resident velocity into s-field VelocityField.

    #[inline]
    pub fn grid(&self) -> GridInfo {
        self.grid
    }

    /// Download GPU velocity field to CPU arrays.
    pub fn download(&self, context: &MetalContext) -> VelocityField {
        let nx = self.grid.nx() as usize;
        let ny = self.grid.ny() as usize;
        let nz = self.grid.nz() as usize;

        let grid = s_field::Grid4D::new(
            [1, nz, ny, nx],
            [1.0, self.grid.dz(), self.grid.dy(), self.grid.dx()],
        );

        //
        // GPU -> CPU
        //
        let u = self.u.download(context);
        let v = self.v.download(context);
        let w = self.w.download(context);

        //
        // Vec -> ScalarField
        //
        let mut u_field = s_field::ScalarField::<f32>::zeros(grid.clone());

        let mut v_field = s_field::ScalarField::<f32>::zeros(grid.clone());

        let mut w_field = s_field::ScalarField::<f32>::zeros(grid.clone());

        u_field.replace(u);
        v_field.replace(v);
        w_field.replace(w);

        VelocityField::from_fields(grid, u_field, v_field, w_field)
    }
}
