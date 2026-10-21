# Render delay and FPS cap

- Ocean simulation advances on a dedicated worker thread; the window event loop no longer performs solver work synchronously.
- Render input snapshots are published at approximately 10 Hz and held in a bounded queue.
- The renderer displays snapshots after a 3-second wall-clock delay. The time finder and flow particles follow the delayed display state; the window title reports the latest calculated state.
- The main event loop requests frames at 10 ms intervals (target maximum 100 FPS).
- The renderer uses `wgpu::PresentMode::AutoNoVsync` so FIFO 60 Hz vsync does not cap the frame rate.
- The time-finder restore action sends a restore command to the simulation worker and clears pending render snapshots.

Build/runtime validation has not been performed in this environment. Verify on the target Mac with `cargo check --workspace` and then run the simulator.


## Streamline density update

- Seed streamlines at every depth layer in the configured 250 m vertical grid.
- Preserve seeds at the original horizontal grid nodes and add one seed at each horizontal grid-cell centre, increasing coverage to roughly 1.9x without the 4x cost of halving both horizontal spacings.
- Reject seeds outside the ocean bounds or below the local bathymetry.
- Compilation/runtime validation still needs to be performed on macOS with Cargo.
