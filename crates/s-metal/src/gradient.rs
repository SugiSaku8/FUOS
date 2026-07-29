use crate::GpuField3D;

pub struct Gradient3D {
    pub dx: GpuField3D,

    pub dy: GpuField3D,

    pub dz: GpuField3D,
}
