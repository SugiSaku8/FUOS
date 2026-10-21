//! Ocean solver using Metal compute.

use crate::{
    AdvectionOperator, BoundaryOperator, BottomDragOperator, BuoyancyOperator, CoriolisOperator, DiffusionOperator,
    DivergenceOperator, GpuField3D, GpuOceanState, GradientPressureOperator, GridInfo,
    LandMaskOperator, MetalContext, PressureSolver,
};
use std::sync::Arc;
use s_field::VelocityField;

pub struct OceanSolver {
    context: Arc<MetalContext>,

    boundary: BoundaryOperator,
    buoyancy: BuoyancyOperator,
    coriolis: CoriolisOperator,
    coriolis_by_latitude: Vec<f32>,
    land_mask_operator: LandMaskOperator,
    bottom_drag: BottomDragOperator,
    land_mask: GpuField3D,
    land_mask_values: Vec<f32>,
    horizontal_viscosity_m2_s: f32,
    vertical_viscosity_m2_s: f32,
    bottom_drag_coefficient: f32,

    advection: AdvectionOperator,

    diffusion: DiffusionOperator,

    divergence: DivergenceOperator,

    pressure: PressureSolver,

    gradient_pressure: GradientPressureOperator,
    use_kuroshio_inflow: bool,
}
impl OceanSolver {
    pub fn new(context: Arc<MetalContext>) -> Self {
        Self::new_with_kuroshio_inflow(context, true)
    }

    /// Construct a solver with an optional simplified Kuroshio inflow boundary.
    pub fn new_with_kuroshio_inflow(context: Arc<MetalContext>, use_kuroshio_inflow: bool) -> Self {
        Self::build(context, use_kuroshio_inflow, None, None, None, 10.0, 1.0e-4, 2.5e-3)
    }

    /// Construct with the source-data boundary, geographic latitude and wet mask.
    pub fn new_with_initial_boundary(
        context: Arc<MetalContext>, use_kuroshio_inflow: bool, initial_velocity: &VelocityField,
        latitude_degrees: &[f64], wet_mask: &[f32],
        horizontal_viscosity_m2_s: f32, vertical_viscosity_m2_s: f32, bottom_drag_coefficient: f32,
    ) -> Self {
        Self::build(context, use_kuroshio_inflow, Some(initial_velocity), Some(latitude_degrees), Some(wet_mask),
            horizontal_viscosity_m2_s, vertical_viscosity_m2_s, bottom_drag_coefficient)
    }

    fn build(
        context: Arc<MetalContext>, use_kuroshio_inflow: bool, initial_velocity: Option<&VelocityField>,
        latitude_degrees: Option<&[f64]>, wet_mask: Option<&[f32]>,
        horizontal_viscosity_m2_s: f32, vertical_viscosity_m2_s: f32, bottom_drag_coefficient: f32,
    ) -> Self {
        let boundary = match initial_velocity {
            Some(velocity) => BoundaryOperator::from_velocity(&context, velocity),
            None => BoundaryOperator::new(&context),
        };
        let (grid, ny, nz, nx) = if let Some(velocity) = initial_velocity {
            let shape = velocity.shape(); let spacing = velocity.spacing();
            (GridInfo::new(shape[2] as u32, shape[1] as u32, shape[0] as u32,
                spacing[3], spacing[2], spacing[1]), shape[1], shape[0], shape[2])
        } else {
            // The compatibility constructor has no field dimensions yet. A 1-cell
            // placeholder mask is replaced by the zero mask path only in test/examples.
            (GridInfo::new(1, 1, 1, 1.0, 1.0, 1.0), 1, 1, 1)
        };
        let n = grid.count();
        let mask_values = wet_mask.filter(|m| m.len() == n).map(|m| m.to_vec()).unwrap_or_else(|| vec![0.0; n]);
        let mask_field = GpuField3D::from_f32(&context, &mask_values, grid);
        let latitude_f = latitude_degrees.filter(|v| v.len() == ny)
            .map(|values| values.iter().map(|lat| crate::coriolis::coriolis_parameter(*lat)).collect())
            .unwrap_or_else(|| vec![0.0; ny]);
        let _ = (nz, nx);
        let buoyancy = BuoyancyOperator::new(&context);
        let coriolis = CoriolisOperator::new(&context);
        let land_mask_operator = LandMaskOperator::new(&context);
        let bottom_drag = BottomDragOperator::new(&context);
        let advection = AdvectionOperator::new(&context);
        let diffusion = DiffusionOperator::new(&context);
        let divergence = DivergenceOperator::new(&context);
        let pressure = PressureSolver::new(&context);
        let gradient_pressure = GradientPressureOperator::new(&context);
        Self { context, boundary, buoyancy, coriolis, coriolis_by_latitude: latitude_f,
            land_mask_operator, bottom_drag, land_mask: mask_field, land_mask_values: mask_values,
            horizontal_viscosity_m2_s: horizontal_viscosity_m2_s.max(0.0),
            vertical_viscosity_m2_s: vertical_viscosity_m2_s.max(0.0),
            bottom_drag_coefficient: bottom_drag_coefficient.clamp(0.0, 0.1),
            advection, diffusion, divergence, pressure, gradient_pressure, use_kuroshio_inflow }
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
        self.step_with_tracers(state, dt, &[], &[]);
    }

    /// Advance momentum with temperature/salinity-dependent Boussinesq buoyancy.
    pub fn step_with_tracers(&self, state: &mut GpuOceanState, dt: f32, temperature: &[f32], salinity: &[f32]) {
        // Density is computed once per step on the CPU from the already-resident
        // tracer arrays. This is small relative to the 3-D field and avoids duplicating
        // the equation-of-state polynomial in several Metal kernels.
        let grid = state.velocity.grid();
        // Legacy solver constructors do not receive geometry; build an all-wet fallback
        // mask lazily when their first actual grid arrives.
        let fallback_mask = vec![0.0_f32; grid.count()];
        let mask_values = if self.land_mask_values.len() == grid.count() {
            self.land_mask_values.as_slice()
        } else {
            fallback_mask.as_slice()
        };
        let fallback_mask_field = if self.land_mask.grid().size() != grid.size() {
            Some(GpuField3D::from_f32(&self.context, mask_values, grid))
        } else { None };
        let mask_field = fallback_mask_field.as_ref().unwrap_or(&self.land_mask);
        let (density, reference_density) = if temperature.len() == grid.count()
            && salinity.len() == temperature.len()
        {
            crate::equation_of_state::density_profile(
                temperature, salinity, grid.nx() as usize, grid.ny() as usize,
                grid.nz() as usize, grid.dz(),
            )
        } else {
            (vec![1025.0_f32; grid.count()], vec![1025.0_f32; grid.nz() as usize])
        };

        // 1. Nonlinear momentum advection.
        state.velocity = self.advection.compute_ssprk2(&self.context, &state.velocity, dt);

        // 2. Latitude-dependent Coriolis rotation, integrated exactly for this substep.
        state.velocity = self.coriolis.compute(&self.context, &state.velocity, &self.coriolis_by_latitude, dt);

        // 3. Molecular viscosity. This is intentionally not presented as a turbulence
        // closure; a validated vertical-mixing closure is still required for ROMS-like skill.
        state.velocity = self.diffusion.compute_velocity_anisotropic(
            &self.context, &state.velocity, self.horizontal_viscosity_m2_s,
            self.vertical_viscosity_m2_s, dt,
        );
        state.velocity = self.bottom_drag.apply(
            &self.context, &state.velocity, mask_field, dt, self.bottom_drag_coefficient,
        );

        // 4. Boussinesq buoyancy from density anomaly relative to the horizontal mean
        // of each depth layer. A horizontally uniform stratification does not create
        // spurious vertical acceleration.
        if temperature.len() == grid.count() && salinity.len() == temperature.len() {
            state.velocity = self.buoyancy.compute_density(
                &self.context, &state.velocity, &density, &reference_density, dt,
            );
        }

        // 5. Prescribed source-data boundary and wet-cell mask before projection.
        if self.use_kuroshio_inflow {
            self.boundary.apply_velocity(&self.context, &mut state.velocity);
        }
        state.velocity = self.land_mask_operator.apply(&self.context, &state.velocity, mask_field);

        // 6. Variable-density projection. With z-index positive downward and w positive
        // upward, divergence and vertical pressure-gradient signs must be opposite.
        let divergence = self.divergence.compute_masked(&self.context, &state.velocity, mask_values);
        state.pressure = self.pressure.solve_variable_density_masked(
            &self.context, &divergence, dt, &density, mask_values, 80,
        );
        state.velocity = self.gradient_pressure.compute_variable_density(
            &self.context, &state.velocity, &state.pressure, dt, &density, mask_values,
        );

        // Reapply prescribed velocity after projection. This is numerically necessary,
        // although a production model should use open-boundary radiation/nudging conditions.
        if self.use_kuroshio_inflow {
            self.boundary.apply_velocity(&self.context, &mut state.velocity);
        }
        state.velocity = self.land_mask_operator.apply(&self.context, &state.velocity, mask_field);
    }
}
