use crate::{GpuField3D, GridInfo, MetalContext};

use s_field::VelocityField;

use crate::slice::field4d_time_slice;

pub struct GpuVelocityField {
    pub u: GpuField3D,

    pub v: GpuField3D,

    pub w: GpuField3D,
}

impl GpuVelocityField {
    pub fn from_velocity_field(
        context: &MetalContext,
        velocity: &VelocityField,
        time: usize,
        grid: GridInfo,
    ) -> Self {
        let u_data = field4d_time_slice(velocity.u(), time).expect("invalid velocity time");

        let v_data = field4d_time_slice(velocity.v(), time).expect("invalid velocity time");

        let w_data = field4d_time_slice(velocity.w(), time).expect("invalid velocity time");

        Self {
            u: GpuField3D::from_f32(context, &u_data, grid),

            v: GpuField3D::from_f32(context, &v_data, grid),

            w: GpuField3D::from_f32(context, &w_data, grid),
        }
    }
}
