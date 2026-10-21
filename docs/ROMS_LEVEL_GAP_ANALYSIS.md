# Toward ROMS-comparable Suruga Bay prediction

## Scope of this implementation stage

The present implementation improves several foundations (equation of state, variable-density pressure projection, vertical sign conventions, Coriolis rotation, bottom drag, masking, source-data boundary, and timestep subcycling). It does not establish ROMS-level forecast skill.

## Why source and grid quality dominate

The current source velocity/tracer grid has only 13 longitude points by 22 latitude points and one source time slice in the configured data pipeline. Vertical values outside the source depth range are extrapolated/attenuated. A single snapshot cannot constrain the temporal evolution of the Kuroshio or its meander. A high-order solver cannot recover information absent from the forcing and initial conditions.

For a meaningful Suruga Bay hindcast, the next data work is to ingest a time series of HYCOM or observational/reanalysis fields, preserve actual bathymetry, use realistic open-boundary fields, and define a fixed validation period that is not used for parameter tuning.

## Required model architecture work

1. **Grid and conservation** — migrate the core to a staggered C-grid or implement a demonstrably conservative finite-volume formulation with face transports, bathymetry masks, and spherical/curvilinear metrics.
2. **Free surface** — add surface elevation, depth-integrated transport, barotropic mode, and stable barotropic subcycling.
3. **Thermodynamics** — move from EOS-80 to TEOS-10-compatible inputs/properties, consistent hydrostatic pressure, and buoyancy/pressure coupling.
4. **Mixing** — implement and validate vertical turbulence closure and horizontal viscosity/diffusivity appropriate to the grid resolution.
5. **Open boundaries** — use time-dependent data, radiation/nudging, transport consistency, and volume conservation rather than a fixed initial boundary face.
6. **Pressure solve** — add residual-based convergence and a multigrid or preconditioned Krylov solver; fixed Jacobi iterations should not be the long-term pressure solver.
7. **Forcing** — add wind stress, heat/freshwater fluxes, tides and rivers when supported by data.
8. **Verification and validation** — unit tests for operators, manufactured-solution convergence tests, mass/heat/salt budgets, time-step and grid refinement, then independent comparison with ROMS or observations.

## Performance strategy

Keep the CPU implementation as a reference path for numerical tests, use Metal for 3-D stencil kernels, avoid per-iteration synchronization, reuse buffers, and profile end-to-end time rather than only kernel time. Any GPU optimization must preserve the CPU reference's numerical tolerances and conservation metrics.
