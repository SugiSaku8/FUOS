use s_metal::{GpuField3D, GpuVelocityField};

pub struct OceanRenderData {
    pub velocity: GpuVelocityField,

    pub temperature: Option<GpuField3D>,

    pub salinity: Option<GpuField3D>,

    pub sea_surface_height: Option<GpuField3D>,
}

impl OceanRenderData {
    pub fn new(velocity: GpuVelocityField) -> Self {
        Self {
            velocity,
            temperature: None,
            salinity: None,
            sea_surface_height: None,
        }
    }
}
