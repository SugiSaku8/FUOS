//! Ocean solver using Metal compute.
//!
//! Navier-Stokes:
//!
//! ∂u/∂t + (u・∇)u
//! = -1/ρ ∇p + ν∇²u

use crate::{
    AdvectionOperator, BoundaryOperator, DiffusionOperator, DivergenceOperator, GpuVelocityField,
    GradientPressureOperator, LandMaskOperator, MetalContext, PressureSolver,
};

use s_field::OceanField;

pub struct OceanSolver {
    context: MetalContext,

    advection: AdvectionOperator,

    diffusion: DiffusionOperator,

    divergence: DivergenceOperator,

    pressure: PressureSolver,

    gradient_pressure: GradientPressureOperator,

    boundary: BoundaryOperator,

    land_mask: LandMaskOperator,
}

impl OceanSolver {
    pub fn new(context: MetalContext) -> Self {
        let advection = AdvectionOperator::new(&context);

        let diffusion = DiffusionOperator::new(&context);

        let divergence = DivergenceOperator::new(&context);

        let pressure = PressureSolver::new(&context);

        let gradient_pressure = GradientPressureOperator::new(&context);

        let boundary = BoundaryOperator::new(&context);

        let land_mask = LandMaskOperator::new(&context);
        Self {
            context,

            advection,

            diffusion,

            divergence,

            pressure,

            gradient_pressure,

            boundary,

            land_mask,
        }
    }

    /// Advance ocean state by dt.
    ///
    /// Navier-Stokes step:
    ///
    /// 1. Advection
    /// 2. Diffusion
    /// 3. Projection
    ///
    pub fn step(&self, ocean: &mut OceanField, dt: f32) {
        //
        // CPU -> GPU
        //
        let mut velocity = GpuVelocityField::from_velocity(&self.context, ocean.velocity());

        //
        // 1. Advection
        //
        velocity = self.advection.compute(&self.context, &velocity, dt);

        //
        // 2. Diffusion
        //
        let viscosity = 1.0e-6_f32;

        velocity = self
            .diffusion
            .compute_velocity(&self.context, &velocity, viscosity, dt);

        //
        // 3. Divergence
        //
        let divergence = self.divergence.compute(&self.context, &velocity);

        //
        // 4. Pressure solve
        //
        let pressure = self.pressure.solve(&self.context, &divergence, 40);

        //
        // 5. Pressure gradient correction
        //
        velocity = self
            .gradient_pressure
            .compute(&self.context, &velocity, &pressure, dt, 1025.0);

        let velocity = self.boundary.apply(
            &self.context,
            &velocity,
            1.0, // 黒潮流入（仮）
            0.0,
            0.0,
        );

        velocity = self
            .land_mask
            .apply(&self.context, &velocity, &gpu_land_mask);

        //
        // GPU -> CPU
        //

        let u = velocity.u.download(&self.context);

        let v = velocity.v.download(&self.context);

        let w = velocity.w.download(&self.context);

        //
        // Write back
        //
        let field_velocity = ocean.velocity_mut();

        field_velocity
            .u_mut()
            .time_slice_mut(0)
            .unwrap()
            .copy_from_slice(&u);

        field_velocity
            .v_mut()
            .time_slice_mut(0)
            .unwrap()
            .copy_from_slice(&v);

        field_velocity
            .w_mut()
            .time_slice_mut(0)
            .unwrap()
            .copy_from_slice(&w);
    }
}
