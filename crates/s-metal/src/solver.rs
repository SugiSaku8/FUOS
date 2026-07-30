//! Ocean solver using Metal compute.

use crate::{
    AdvectionOperator, DiffusionOperator, DivergenceOperator, GpuVelocityField,
    GradientPressureOperator, MetalContext, PressureSolver,
};

use s_field::OceanField;

pub struct OceanSolver {
    context: MetalContext,

    advection: AdvectionOperator,

    diffusion: DiffusionOperator,

    divergence: DivergenceOperator,

    pressure: PressureSolver,

    gradient_pressure: GradientPressureOperator,
}

impl OceanSolver {
    pub fn new(context: MetalContext) -> Self {
        let advection = AdvectionOperator::new(&context);

        let diffusion = DiffusionOperator::new(&context);

        let divergence = DivergenceOperator::new(&context);

        let pressure = PressureSolver::new(&context);

        let gradient_pressure = GradientPressureOperator::new(&context);

        Self {
            context,

            advection,

            diffusion,

            divergence,

            pressure,

            gradient_pressure,
        }
    }

    /// Advance ocean state.
    ///
    /// Navier-Stokes simplified step.
    ///
    ///
    /// ∂u/∂t
    ///
    /// +
    ///
    /// (u・∇)u
    ///
    /// =
    ///
    /// -1/rho ∇p
    ///
    /// +
    ///
    /// ν∇²u
    ///
    pub fn step(&self, ocean: &mut OceanField, dt: f32) {
        //
        // CPU -> GPU
        //
        let velocity = GpuVelocityField::from_velocity(&self.context, ocean.velocity());

        //
        // 1. Advection
        //
        let velocity = self.advection.compute(&self.context, &velocity, dt);

        //
        // 2. Diffusion
        //
        let velocity = self
            .diffusion
            .compute_velocity(&self.context, &velocity, 0.000001, dt);

        //
        // 3. Divergence
        //
        let divergence = self.divergence.compute(&self.context, &velocity);

        //
        // 4. Pressure Poisson solve
        //
        let pressure = self.pressure.solve(&self.context, &divergence, 40);

        //
        // 5. Pressure gradient correction
        //
        let velocity =
            self.gradient_pressure
                .compute(&self.context, &velocity, &pressure, dt, 1025.0);

        //
        // GPU -> CPU
        //

        let u = velocity.u.download(&self.context);

        let v = velocity.v.download(&self.context);

        let w = velocity.w.download(&self.context);

        let field = ocean.velocity_mut();

        field.u_mut().time_slice_mut(0).unwrap().copy_from_slice(&u);

        field.v_mut().time_slice_mut(0).unwrap().copy_from_slice(&v);

        field.w_mut().time_slice_mut(0).unwrap().copy_from_slice(&w);
    }
}
