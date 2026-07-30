use s_field::{
    Coordinates, Grid4D, Metadata, OceanField, SalinityField, TemperatureField, VelocityField,
};

use s_metal::{MetalContext, OceanSolver};

fn main() {
    //
    // Metal context
    //
    let context = MetalContext::new();

    //
    // Grid
    //
    let nx = 16;
    let ny = 16;
    let nz = 16;
    let nt = 1;

    let grid = Grid4D::new([nx, ny, nz, nt], [1.0, 1.0, 1.0, 1.0]);

    //
    // Coordinates
    //
    let coordinates =
        Coordinates::new(grid, vec![0.0; nx], vec![0.0; ny], vec![0.0; nz], vec![0.0]);

    //
    // Metadata
    //
    let metadata = Metadata::default();

    //
    // Ocean fields
    //
    let velocity = VelocityField::new(grid);

    let temperature = TemperatureField::new(grid);

    let salinity = SalinityField::new(grid);

    //
    // OceanField
    //
    let mut ocean = OceanField::new(coordinates, metadata, velocity, temperature, salinity);

    //
    // Initial velocity
    //
    let cx = nx / 2;
    let cy = ny / 2;
    let cz = nz / 2;

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let dx = x as f32 - cx as f32;

                let dy = y as f32 - cy as f32;

                let dz = z as f32 - cz as f32;

                let r2 = dx * dx + dy * dy + dz * dz;

                let value = (-r2 / 20.0).exp();

                let i = z * nx * ny + y * nx + x;

                //
                // u velocity
                //
                {
                    let u = ocean.velocity_mut().u_mut().time_slice_mut(0).unwrap();

                    u[i] = value;
                }

                //
                // v velocity
                //
                {
                    let v = ocean.velocity_mut().v_mut().time_slice_mut(0).unwrap();

                    v[i] = 0.0;
                }

                //
                // w velocity
                //
                {
                    let w = ocean.velocity_mut().w_mut().time_slice_mut(0).unwrap();

                    w[i] = 0.0;
                }
            }
        }
    }

    //
    // CPU initial check
    //
    let before = ocean.velocity().u().time_slice(0).unwrap();

    println!(
        "CPU before center = {}",
        before[cz * nx * ny + cy * nx + cx]
    );

    //
    // Solver
    //
    let solver = OceanSolver::new(context);

    solver.step(&mut ocean, 0.01);

    //
    // CPU result
    //
    let after = ocean.velocity().u().time_slice(0).unwrap();

    println!("CPU after center = {}", after[cz * nx * ny + cy * nx + cx]);

    //
    // Neighborhood
    //
    println!("neighborhood:");

    for z in cz - 1..=cz + 1 {
        for y in cy - 1..=cy + 1 {
            for x in cx - 1..=cx + 1 {
                let i = z * nx * ny + y * nx + x;

                println!("{} {} {} : {}", x, y, z, after[i]);
            }
        }
    }
}
