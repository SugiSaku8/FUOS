# SOS simulator implementation status

This file distinguishes implemented numerical changes from features that are still required before the model can be described as ROMS-comparable.

## Implemented in this stage

- Restored the `SimulationEngine` read-only accessors used by `render.rs`: `terrain`, `field`, `is_finished`, `step`, and `time_seconds`.
- Added UNESCO EOS-80 seawater density at atmospheric pressure plus a hydrostatic compressibility correction based on depth. Invalid/masked tracer cells are excluded from layer reference-density means.
- Applied Boussinesq buoyancy using density anomaly relative to the horizontal mean of each depth layer, avoiding a spurious vertical acceleration caused by a constant reference temperature/salinity.
- Changed pressure projection to the variable-density form `div((1/rho) grad(p)) = div(u*) / dt` and changed the pressure correction to use face-averaged inverse density.
- Made pressure boundaries Neumann-like and fixed the pressure gauge at a wet cell. Land cells are excluded from pressure-neighbour coupling.
- Corrected the vertical sign convention consistently: depth index increases downward, while `w` is positive upward.
- Added latitude-dependent Coriolis rotation using an exact horizontal-velocity rotation for the Coriolis-only substep.
- Added a quadratic bottom-drag approximation on deepest wet cells.
- Added a wet-cell mask to velocity, divergence, pressure and pressure-gradient processing. The current data builder's `(T,S)=(0,0)` land/sub-bottom marker is used to build this mask.
- Replaced the simulator's hard-coded synthetic inflow with a fixed boundary sampled from the input velocity field. The legacy synthetic boundary remains only for standalone solver examples.
- Added SSP-RK2 integration for the momentum-advection term and CFL-based momentum subcycling. Scalar transport now also subcycles for advective/diffusive stability.
- Added configurable horizontal/vertical background viscosity and bottom-drag coefficient.
- Added a CPU reference backend (`s-cpu`) with the same equation-of-state function, Coriolis rotation, advection integrator, bottom drag, wet mask and variable-density pressure projection; select it with `--cpu`.
- Encoded pressure Jacobi passes in one Metal command buffer, avoiding a CPU/GPU wait after every iteration.

## Important remaining gaps

This is still **not a ROMS-equivalent model**. The code is a lightweight collocated-grid nonhydrostatic prototype. It does not yet implement the defining ROMS architecture and parameterizations:

1. A terrain-following curvilinear Arakawa C-grid, metric terms and conservative fluxes.
2. Prognostic free-surface elevation and barotropic/baroclinic mode splitting.
3. A validated open-boundary radiation/nudging scheme and time-varying HYCOM/observational forcing. The currently prescribed boundary is fixed to the initial data face.
4. A turbulence closure such as GLS or K-profile parameterization, with calibrated vertical mixing.
5. Full bathymetry-aware face masks and consistent finite-volume fluxes at coastlines. The present mask reduces land leakage but is not a complete wetting/drying or C-grid coastline treatment.
6. TEOS-10 thermodynamics (including Absolute Salinity, Conservative Temperature and pressure-dependent Gibbs-function properties). EOS-80 is used here as an approximation.
7. A residual-controlled pressure solver/preconditioner. The current fixed-iteration Jacobi method is lightweight but may not converge sufficiently for larger or strongly anisotropic grids.
8. Surface heat/freshwater fluxes, wind stress, tides, river discharge and data-assimilative initial/boundary conditions.
9. A reproducible skill assessment against withheld observations or a trusted reference run, including velocity RMSE/correlation, sea-surface height, temperature/salinity profiles, conservation and grid/time-step convergence.

The vertical boundary currently applies a **rigid-lid/no-normal-flow approximation** because no prognostic sea-surface elevation is implemented. It must not be interpreted as a free-surface model. The CPU solver is intended as a correctness/reference backend; it is serial and may be slower than Metal for large grids.

## Configuration defaults introduced

- `horizontal_viscosity_m2_s`: `10.0`
- `vertical_viscosity_m2_s`: `1e-4`
- `bottom_drag_coefficient`: `0.0025`
- `scalar_diffusivity_m2_s`: `1e-5`

These are background numerical/physical parameters, not regionally calibrated values. They should be sensitivity-tested rather than treated as verified Suruga Bay coefficients.

## Verification status

The editing environment used to prepare this stage has no Rust toolchain and no Apple Metal runtime. Therefore no claim is made that `cargo check`, the Metal shader compiler, runtime tests, or prediction-skill metrics have passed here. Run the commands in `docs/VALIDATE_SOS.md` on the target Apple Silicon Mac before using this build for experiments.
