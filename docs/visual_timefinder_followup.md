# SOS visual/time-finder follow-up patch

## Fixes and changes

- `Field4D<T>` now exposes inherent `as_slice()` and `as_mut_slice()` methods. Downstream crates no longer depend on importing the `Field` trait merely to access contiguous storage. Existing `data()` / `data_mut()` names remain as aliases.
- Added a unit test for the inherent slice API.
- Flow-particle temperature coloring uses the finite-value domain mean as the reference temperature instead of a hard-coded 10 °C. Upward/downward vertical velocity remains the higher-priority diagnostic color. The reference mean is calculated once per particle update, avoiding a full temperature-array scan and allocation for every particle.
- Increased default particle trail length to 48 and flow display multiplier to 240. Seed strides remain 1 in both horizontal directions, which is the densest grid-based default supported by this seeding scheme.
- Increased default FPS fly-through speed to 30,000 world units/s and made mouse-wheel movement use the configured speed.
- Camera movement is clamped to the terrain's horizontal world bounds and the camera is kept at least 250 world units above the sampled bathymetry surface.
- The separate time-finder window and snapshot seeking from the previous package are retained.

## Validation status

The package JSON was parsed and the ZIP archive integrity was checked. This editing environment does not provide `cargo`/`rustc`, so Rust compilation, unit tests, and Metal runtime behavior have not been verified here. On macOS, run from the workspace root:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

## Physical-model caveat

Temperature anomaly colors are visualization diagnostics. The existence of a colored upward-flow particle does not by itself validate local upwelling physics. Validate the solver against conservation tests and observations before interpreting it as a forecast.
