# SOS numerical validation stage — 2026-10-21

## What was executed in this environment

Executed `python3 tools/validation/validate_numerics.py` with Python 3.13.5 and NumPy 2.3.5 on Linux x86_64. All six independent reference checks passed:

| Check | Result | Threshold | Status |
|---|---:|---:|---|
| Relative pressure residual (periodic manufactured Poisson problem, 24^3, 300 Jacobi sweeps) | 3.0418194e-05 | < 1e-3 | PASS |
| Projected RMS divergence ratio (20^3, 500 sweeps) | 6.2594243e-08 | < 0.05 | PASS |
| Periodic scalar mass relative drift (finite-volume flux form, 128 cells, 100 steps) | 0 | < 1e-12 | PASS |
| Smooth advection grid-refinement order | 1.9979 | > 1.7 | PASS |
| SSP-RK2 time-refinement order | 2.0276 | > 1.8 | PASS |
| Smooth advection grid-refinement order at low CFL | 1.9979 | > 1.7 | PASS |

Machine-readable output: `docs/validation/reference_test_results.json`.

## Critical scope limitation

These checks execute independent NumPy reference problems. They do **not** execute the SOS Rust CPU solver, compile Metal shaders, run the simulator, or compare CPU and Metal outputs. The environment has no `rustc` or `cargo`, and it is not a macOS/Metal host. Therefore, the results establish that the reference tests and test criteria behave as expected; they do not establish that the current SOS implementation passes them.

## Source audit findings requiring Mac/device validation

- `crates/s-metal/src/pressure.rs` still uses a fixed iteration count (the solver call site passes 80) and does not compute a residual or stop on a tolerance. A manufactured pressure test should measure the actual residual before a threshold is claimed.
- `crates/s-cpu/src/lib.rs` has a fixed 80-iteration Jacobi loop and does not expose the pressure residual or post-projection divergence as diagnostics.
- The CPU velocity advection uses the advective form `u · grad(q)` with centered differences, not a conservative finite-volume flux form. Therefore the scalar mass-conservation reference test cannot be treated as evidence that SOS conserves volume, heat, or salt.
- The CPU and Metal backends have not been compared on identical fields in this environment. Their current schemes and boundary/mask details must be tested on a Mac.
- The supplied source data is very coarse (13 x 22 horizontal cells in the documented configuration) and a single time slice. It cannot support a claim of ROMS-level forecast skill by itself.

## Run on an Apple Silicon Mac

From the project root:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo test -p s-cpu
cargo run -p simulator -- --cpu --fresh
cargo run -p simulator -- --metal --fresh
```

For the independent numerical reference suite, install NumPy in the Python environment and run:

```bash
python3 tools/validation/validate_numerics.py
```

For the actual model, record and compare on the same small initial field: pressure residual `||A p - rhs|| / ||rhs||`, divergence RMS before/after projection, total wet-cell volume/heat/salt budgets, max CFL, and L2 differences between CPU and Metal. Repeat with `dt`, `dt/2`, `dt/4` and successively refined grids. Do not interpret the reference tests as SOS model verification.

## ROMS-comparability status

**Not established.** To make such a claim, SOS still needs a consistent conservative/staggered-grid core, pressure convergence control, validated boundary conditions, realistic time-dependent forcing, thermodynamic/mixing closure, and blind comparison against observations or a verified ROMS hindcast over the same period.
