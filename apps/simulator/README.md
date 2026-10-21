# SOS Simulator app

`apps/simulator` is the orchestration entry point. Run from the repository root:

```sh
cargo run -p simulator -- apps/simulator/config/simulator.json
```

The JSON config is the single place for source paths, grid/depth expansion policy, numerical time settings, render toggles, and output/checkpoint destinations.

## Intended runtime pipeline

1. Load and validate `simulator.json` and all configured inputs.
2. Load bathymetry and construct the simulation `Grid`/mask.
3. Normalize the raw current/scalar fields to that grid and write a prepared SOF package (`grid.snc`, `boundary.snc`, `initial.snc`, manifest).
4. Initialize `simulation::Simulation` with the Metal context and prepared fields; advance in fixed `dt_seconds` increments.
5. Run the renderer on the window/event-loop thread. The solver publishes immutable snapshots through a bounded latest-value channel; rendering must not block the solver.
6. Render terrain and flow particles/streamlines from the latest simulation snapshot, with depth positions derived from the same configured depth axis and vertical scale.
7. Save immutable step checkpoints under `output.checkpoint_dir`; after normal completion, write one final SOF package under `output.final_sof`.

## Current implementation status

The first commit adds the app config schema, input validation/bootstrap, and an atomic binary checkpoint writer. It deliberately does **not** claim that steps 2–7 are already executing: the current `terrain.rs` example still visualizes a prepared/source velocity field, while the full SOF-to-Metal-to-renderer snapshot bridge is the next integration task. This separation prevents static source data from being mistaken for live solver output.

## Checkpoint format (intermediate)

Each `step_XXXXXXXXXXXX/` directory contains `u.bin`, `v.bin`, `w.bin`, and `metadata.json`. The three fields are little-endian `f32`, flattened in `z × y × x` order. A directory is first written as a temporary staging directory and renamed only after all files are flushed. The final deliverable remains a SOF package, independent of these intermediate snapshots.
