//! Ocean solver using Metal compute.

use crate::{AdvectionOperator, GpuVelocityField, MetalContext};

use s_field::OceanField;

pub struct OceanSolver {
    context: MetalContext,
    advection: AdvectionOperator,
}

impl OceanSolver {
    pub fn new(context: MetalContext) -> Self {
        let advection = AdvectionOperator::new(&context);

        Self { context, advection }
    }

    /// Advance ocean state by dt.
    ///
    /// Equation:
    ///
    /// ∂u/∂t + (u・∇)u = 0
    ///
    pub fn step(&self, ocean: &mut OceanField, dt: f32) {
        //
        // CPU -> GPU
        //
        let gpu_velocity = GpuVelocityField::from_velocity(&self.context, ocean.velocity());

        //
        // GPU advection
        //
        let result = self.advection.compute(&self.context, &gpu_velocity, dt);

        //
        // GPU -> CPU
        //
        let u = result.u.download(&self.context);

        let v = result.v.download(&self.context);

        let w = result.w.download(&self.context);

        //
        // Copy only t=0 layer
        //
        let velocity = ocean.velocity_mut();

        let u_dst = velocity
            .u_mut()
            .time_slice_mut(0)
            .expect("missing velocity time slice");

        u_dst.copy_from_slice(&u);

        let v_dst = velocity
            .v_mut()
            .time_slice_mut(0)
            .expect("missing velocity time slice");

        v_dst.copy_from_slice(&v);

        let w_dst = velocity
            .w_mut()
            .time_slice_mut(0)
            .expect("missing velocity time slice");

        w_dst.copy_from_slice(&w);
    }
}
