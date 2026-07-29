use crate::{GpuVelocityField, Gradient3D, GradientOperator, MetalContext, VelocityGradient};

pub struct VelocityGradientOperator {
    gradient: GradientOperator,
}

impl VelocityGradientOperator {
    pub fn new(gradient: GradientOperator) -> Self {
        Self { gradient }
    }

    pub fn compute(&self, context: &MetalContext, velocity: &GpuVelocityField) -> VelocityGradient {
        let u_gradient = self.gradient.gradient(context, &velocity.u);

        let v_gradient = self.gradient.gradient(context, &velocity.v);

        let w_gradient = self.gradient.gradient(context, &velocity.w);

        VelocityGradient {
            u_dx: u_gradient.dx,

            u_dy: u_gradient.dy,

            u_dz: u_gradient.dz,

            v_dx: v_gradient.dx,

            v_dy: v_gradient.dy,

            v_dz: v_gradient.dz,

            w_dx: w_gradient.dx,

            w_dy: w_gradient.dy,

            w_dz: w_gradient.dz,
        }
    }
}
