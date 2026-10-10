//! Ocean solver using Metal compute.

use crate::{
    AdvectionOperator, BoundaryOperator, DiffusionOperator, DivergenceOperator, GpuOceanState,
    GradientPressureOperator, MetalContext, PressureSolver,
};
use std::sync::Arc;

pub struct OceanSolver {
    context: Arc<MetalContext>,

    boundary: BoundaryOperator,

    advection: AdvectionOperator,

    diffusion: DiffusionOperator,

    divergence: DivergenceOperator,

    pressure: PressureSolver,

    gradient_pressure: GradientPressureOperator,
}
impl OceanSolver {
    pub fn new(context: Arc<MetalContext>) -> Self {
        let boundary = BoundaryOperator::new(&context);

        let advection = AdvectionOperator::new(&context);

        let diffusion = DiffusionOperator::new(&context);

        let divergence = DivergenceOperator::new(&context);

        let pressure = PressureSolver::new(&context);

        let gradient_pressure = GradientPressureOperator::new(&context);

        Self {
            context,

            boundary,

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
    pub fn step(&self, state: &mut GpuOceanState, dt: f32) {
        //
        // 1. Advection
        //
        state.velocity = self.advection.compute(&self.context, &state.velocity, dt);

        //
        // 2. Diffusion
        //
        state.velocity =
            self.diffusion
                .compute_velocity(&self.context, &state.velocity, 0.000001, dt);

        //
        // 3. Boundary condition
        //
        self.boundary
            .apply_velocity(&self.context, &mut state.velocity);

        //
        // 4. Divergence
        //
        // 速度場の発散を計算
        let divergence = self.divergence.compute(&self.context, &state.velocity);

        // 海水密度の代表値
        let rho = 1025.0_f32;

        // 圧力ポアソン方程式を解く
        state.pressure = self.pressure.solve(&self.context, &divergence, dt, rho, 40);

        // 圧力勾配によって速度場を補正
        state.velocity = self.gradient_pressure.compute(
            &self.context,
            &state.velocity,
            &state.pressure,
            dt,
            rho,
        );

        //
        // 7. Boundary correction
        //
        self.boundary
            .apply_velocity(&self.context, &mut state.velocity);
    }
}
