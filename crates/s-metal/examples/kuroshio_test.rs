use std::sync::Arc;

use s_field::{BoundaryField, Coordinates, Grid4D, Metadata, OceanField, SalinityField, TemperatureField, VelocityField};
use s_metal::{GpuOceanState, MetalContext, OceanSolver};

fn main() {
    let nx = 32;
    let ny = 32;
    let nz = 16;
    let grid = Grid4D::new([1, nz, ny, nx], [1.0, 1000.0, 1000.0, 1000.0]);
    let coordinates = Coordinates::new(
        grid,
        (0..ny).map(|i| 34.5 + i as f64 * 0.01).collect(),
        (0..nx).map(|i| 138.5 + i as f64 * 0.01).collect(),
        (0..nz).map(|i| i as f32 * 50.0).collect(),
        vec![0.0],
    );
    let mut velocity = VelocityField::new(grid);
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let i = z * nx * ny + y * nx + x;
                if y < 4 {
                    velocity.u_mut().as_mut_slice()[i] = 0.5;
                    velocity.v_mut().as_mut_slice()[i] = 0.15;
                }
            }
        }
    }
    velocity.set_coordinates(coordinates.clone());
    let field = OceanField::new(
        coordinates, Metadata::default(), velocity, TemperatureField::new(grid),
        SalinityField::new(grid), BoundaryField::zeros(grid),
    );
    let context = Arc::new(MetalContext::new());
    let solver = OceanSolver::new_with_kuroshio_inflow(context.clone(), true);
    let mut state = GpuOceanState::from_ocean(&context, &field);
    for step in 0..10 {
        solver.step(&mut state, 0.01);
        println!("step {} completed", step + 1);
    }
    let snapshot = state.velocity.snapshot(&context);
    println!("velocity cells={} center_u={:.6}", snapshot.size(), snapshot.u[snapshot.index(nx / 2, ny / 2, nz / 2)]);
}
