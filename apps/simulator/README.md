# Suruga Ocean Simulator (`apps/simulator`)

The simulator coordinates source-data validation, grid preparation, SOF serialization, a CPU reference solver or Apple Metal solver, live WGPU visualization, checkpointing and final output.

## Run

From the workspace root on macOS with a Rust toolchain:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p simulator -- --help
```

Default backend is controlled by `simulation.use_metal` in `apps/simulator/config/simulator.json` (Metal in the supplied config). Override for one invocation:

```bash
cargo run -p simulator -- --metal --fresh
cargo run -p simulator -- --cpu --fresh
```

- `--fresh`: ignore checkpoints and start from configured source inputs.
- `--resume`: resume from the latest valid checkpoint in the configured output root.
- `--cpu`: use the serial CPU reference implementation.
- `--metal`: use Apple Metal compute.

## Pipeline

1. Read the config, terrain grid SNC, source coordinates JSON and little-endian f32 U/V/temperature/salinity files.
2. Build a solver grid with the configured vertical interval and mask land/sub-bottom cells.
3. Create `input.sof`, including solver grid, high-resolution terrain, boundary, initial, restart and forcing datasets; read the initial field back before initializing the solver.
4. Advance momentum with SSP-RK2 advection, latitude-dependent Coriolis rotation, configurable anisotropic background viscosity, quadratic bottom drag, EOS-80-based density anomaly buoyancy, wet-cell masking and variable-density pressure projection. Momentum substeps are chosen from an estimated advective CFL.
5. Advance temperature and salinity with TVD minmod-limited gradients, SSP-RK2 and stability-limited explicit diffusion.
6. If rendering is enabled, update the WGPU terrain and particle-flow view from the latest CPU-visible field.
7. Publish checkpoints and final SOF output through staged writes.

## Configuration notes

- `horizontal_viscosity_m2_s`: background horizontal viscosity; default `10.0`.
- `vertical_viscosity_m2_s`: background vertical viscosity; default `1e-4`.
- `bottom_drag_coefficient`: quadratic bottom-drag coefficient; default `0.0025`.
- `scalar_diffusivity_m2_s`: background tracer diffusivity; default `1e-5`.

These defaults are starting parameters, not calibrated Suruga Bay coefficients. They require sensitivity testing against observations.

## Source data layout

The supplied config expects:

- `suruga_bay.sof/grid.snc`
- `crates/s-builder/suruga_raw/coordinates.json`
- `crates/s-builder/suruga_raw/u.bin`
- `crates/s-builder/suruga_raw/v.bin`
- `crates/s-builder/suruga_raw/temperature.bin`
- `crates/s-builder/suruga_raw/salinity.bin`

Raw arrays are little-endian `f32` in `(depth, latitude, longitude)` order. `coordinates.json` must contain strictly increasing depth, latitude and longitude arrays whose product matches each raw array length.

## Scientific limitations

This is a lightweight collocated-grid prototype, **not a ROMS-equivalent model**. The current source field is coarse (13×22 horizontal points) and has a single time slice; deeper values are extrapolated/attenuated. The prescribed open boundary is fixed to the initial source face. The vertical boundary is a rigid-lid/no-normal-flow approximation; there is no prognostic sea-surface elevation/barotropic mode. The model does not yet have an Arakawa C-grid, a validated turbulence closure, TEOS-10, time-varying radiation/nudging boundaries, complete coastline finite-volume fluxes, tides, wind stress, heat/freshwater fluxes, or independent observation-based skill validation.

The CPU backend is intended for reference calculations and small-grid comparisons. It is serial and will generally be slower than Metal for larger grids. The Metal path is the intended performance backend on Apple Silicon.

See `apps/simulator/IMPLEMENTATION_STATUS.md`, `docs/ROMS_LEVEL_GAP_ANALYSIS.md`, and `docs/VALIDATE_SOS.md` for the implementation boundary and validation procedure.
