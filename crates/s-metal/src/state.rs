use crate::{GpuField3D, GpuVelocityField, MetalContext};

use s_field::OceanField;

pub struct GpuOceanState {
    pub velocity: GpuVelocityField,

    pub pressure: GpuField3D,
}

impl GpuOceanState {
    pub fn from_ocean(context: &MetalContext, ocean: &OceanField) -> Self {
        let velocity = GpuVelocityField::from_velocity(context, ocean.velocity());

        let grid = velocity.grid();

        let pressure = GpuField3D::empty(context, grid);

        Self { velocity, pressure }
    }

    pub fn download_velocity(&self, context: &MetalContext) -> s_field::VelocityField {
        self.velocity.download(context)
    }
}
