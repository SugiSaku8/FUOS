use crate::GpuField3D;

pub struct VelocityGradient {
    pub u_dx: GpuField3D,
    pub u_dy: GpuField3D,
    pub u_dz: GpuField3D,

    pub v_dx: GpuField3D,
    pub v_dy: GpuField3D,
    pub v_dz: GpuField3D,

    pub w_dx: GpuField3D,
    pub w_dy: GpuField3D,
    pub w_dz: GpuField3D,
}
