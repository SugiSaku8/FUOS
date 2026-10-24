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

## Data generation pipeline (Phases 1–7)

- Added `cargo run -p simulator -- generate`, an interactive Y/N wizard that consolidates the raw source arrays and coordinates into a single `.sosdata` file, persists the new config input, and optionally creates the simulator SOF package immediately.
- Added a versioned `SOSSRC02` bundle reader/writer (backward-compatible with `SOSSRC01`) with optional vertical velocity `w`, coordinate/length/finite-value validation, and a round-trip unit test.
- Added `GridBuilder`, which derives the solver grid from source coordinate axes, terrain depth, and the configured vertical spacing. `grid.snc` is generated as an output, not requested as a separate solver-grid input.
- The field builder interpolates selected source fields to the generated vertical grid and applies a terrain-derived bottom/land mask. If a variable is omitted, the builder prints a warning and uses explicit placeholder initial values (zero velocity, 15 °C temperature, 35 PSU salinity); these are not observationally reconstructed values.
- The initial, boundary, restart, forcing, grid, terrain and manifest files are written into the SOF package. Boundary edge arrays are derived from the generated initial velocity field.
- Normal simulator startup consumes `source_bundle` and no longer requires separate coordinate, u, v, temperature and salinity paths. The old individual paths remain supported only as a migration fallback.
- See `docs/DATA_GENERATION_PIPELINE.md` for the bundle format and execution instructions.

## Source diagnostics and shared pipeline (added)

- Added `diagnose` CLI and machine-readable `diagnostics.json` output.
- Checks configured source file existence, SOF terrain shape/axes/spacing/bathymetry, source coordinate ordering/ranges/spacing, bundle validation, field sample counts, finite values, min/max/mean/zero fractions, common physical range outliers, constant fields, missing variables, terrain coverage, and source-depth coverage.
- `ERROR` findings block data generation. The Y/N wizard requires explicit confirmation when only warnings are present. Normal startup performs the same preflight and blocks errors.
- Added `pipeline.rs` as the common orchestration layer. Both the wizard and normal startup now use the same diagnostic -> validation gate -> dataset builder route. Future grid generation, interpolation, masking, initialization, and SOF output improvements should remain behind this shared pipeline.
- The diagnostic is a structural/plausibility check, not scientific certification. Units, source timestamps, provider-specific missing values, source-grid staggering, and model skill still require domain-specific verification.
- Rust compilation/runtime tests were not run in the current environment because `cargo`/`rustc` are unavailable; run `cargo check --workspace` and `cargo test --workspace` on the Apple Silicon Mac.


## CLI mode separation

- `cargo run -p simulator -- generate`: interactive generation workflow.
- `cargo run -p simulator -- diagnose`: diagnostics only.
- `cargo run -p simulator -- --simulation`: simulation only, loading existing `input.sof`.
- `cargo run -p simulator -- --all`: generate, diagnose, then simulate.
- `cargo run -p simulator -- --y`: asks for one initial confirmation; Y auto-accepts generator yes/no prompts, while N/Enter falls back to simulation-only behavior.
- No mode flag defaults to simulation-only.


## Native NetCDF / GeoTIFF input pipeline

- Replaced script-based preprocessing and acquisition with native Rust code in `apps/simulator/src/netcdf_pipeline.rs`.
- NetCDF terrain and ocean fields are read through the Rust `netcdf` crate; geographic GeoTIFF terrain is read through GDAL Rust bindings and requires a north-up geographic grid.
- `cargo run -p simulator -- get --hycom [config.json] [--force]` retrieves a regional NetCDF response with `curl` from the configured public HYCOM NCSS endpoint.
- The native pipeline writes the terrain via `s-sof::GridWriter`, writes the normalized ocean source bundle, and keeps final SOF package creation in the shared Rust pipeline.
- All external preprocessing scripts, interpreter setup, and package requirements were removed. Rust compilation and live HYCOM retrieval must be verified on the target Mac.
