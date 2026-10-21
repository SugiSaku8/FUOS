# SOS engineering notes — 2026-10-21

## Repair
`apps/simulator/src/engine.rs` had an incomplete `SimulationEngine` implementation: scalar advancement and save routines were called but absent, and the `impl` block was not closed before free numerical helper functions. These methods are restored and the impl is closed after `sync_cpu_field`.

## Temperature/salinity coupling
`crates/s-metal/shaders/buoyancy.metal` adds a vertical Boussinesq buoyancy acceleration:

`b = g * (alpha * (T - 10 °C) - beta * (S - 35 PSU))`

with `alpha = 2e-4 1/°C`, `beta = 7.6e-4 1/PSU`, and positive vertical velocity defined as upward. `OceanSolver::step_with_tracers` applies this kernel to the current scalar fields before divergence and pressure projection. Existing `step` remains available for callers without tracers.

## Limits
This does not yet solve a fully variable-density pressure equation. The pressure solve and pressure-gradient correction still use a representative density of 1025 kg/m³; momentum advection remains forward Euler; no Coriolis, free-surface, validated turbulence closure, or time-varying observational boundary forcing has been added. Treat the new coupling as an incremental physical improvement, not evidence of forecast accuracy. Build and test on the target Mac with `cargo check -p simulator` and `cargo test -p simulator`.
