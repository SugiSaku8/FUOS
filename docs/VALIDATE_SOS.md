# Validate SOS on an Apple Silicon Mac

Run from the repository root after extracting the archive.

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p simulator -- --help
cargo test -p s-cpu
```

To compare backends on the same configured input, run each backend from a fresh checkpoint state:

```bash
cargo run -p simulator -- --cpu --fresh
cargo run -p simulator -- --metal --fresh
```

The CPU backend is a serial reference implementation. Use it for numerical cross-checks on small cases; it is not intended to out-perform Metal on large grids.

The simulator's Metal kernels require macOS and a Metal-capable device. A successful Linux `cargo check` would not validate Metal shader compilation or runtime behaviour.

## Numerical diagnostics

Run the focused projection and operator examples on the Mac:

```bash
cargo run -p s-metal --example projection_test
cargo run -p s-metal --example divergence_test
cargo run -p s-metal --example advection_test
cargo run -p s-metal --example diffusion_test
cargo run -p s-metal --example pressure_test
```

Record, at minimum:

- divergence RMS before and after projection;
- pressure-equation residual, not just the iteration count;
- minimum/maximum density, temperature and salinity;
- maximum CFL number and the number of substeps;
- total simulation time, GPU time, CPU/GPU synchronization count and peak memory;
- conservation drift for volume, heat and salt;
- velocity/temperature/salinity error against an independent observation/reference period.

Do not label the run ROMS-equivalent unless it passes predeclared quantitative validation thresholds on data not used to tune the model.


## 2026-10-21 renderer / CPU fixes

- `s-cpu` imports the `s_field::Field` trait required for `Field4D::as_slice()`.
- Flow vertices carry per-vertex RGB. Particle trails are colored by local temperature anomaly; upward vertical velocity is highlighted green and downward motion orange. The solver's buoyancy coupling remains the physical source of temperature-driven vertical motion; colors are diagnostics, not a separate physical force.
- Default rendering seeds every horizontal velocity-grid point at each wet depth level (`seed_stride_x/y = 1`) and keeps 36 trail points. The default particle animation multiplier is 220 and the simulation advances 4 model steps per frame. These settings increase visual density and simulation-time progression, and may be reduced on lower-end machines.
- The FPS camera moves faster and clamps its Y position above the local bathymetry to reduce terrain clipping.
- A separate Time Finder window draws a timeline. Clicking the bar seeks to the nearest retained snapshot. Snapshots are retained in a rolling history (up to 180 states); seeking only covers retained history, not the entire run. Restoring a snapshot rebuilds the Metal state from the CPU-side snapshot.

The current environment has no Rust/Cargo toolchain or Apple Metal runtime, so these changes have not been compiled or exercised here. Run `cargo fmt --all`, `cargo check --workspace`, and `cargo test --workspace` on macOS before relying on the build.
