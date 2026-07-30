use s_field::{
    Coordinates, Grid4D, Metadata, OceanField, SalinityField, TemperatureField, VelocityField,
};

use s_metal::{MetalContext, OceanSolver};

fn main() {
    //
    // Grid
    //
    let grid = Grid4D::new([32, 32, 16, 1], [1000.0, 1000.0, 50.0, 1.0]);

    let coordinates = Coordinates::new(
        grid.clone(),
        vec![0.0; 32],
        vec![0.0; 32],
        vec![0.0; 16],
        vec![0.0],
    );

    let metadata = Metadata::default();

    //
    // Velocity
    //
    let mut velocity = VelocityField::new(grid.clone());

    //
    // 黒潮初期条件
    //
    {
        let nx = 32;
        let ny = 32;
        let nz = 16;

        //
        // u component
        //
        {
            let u = velocity.u_mut().time_slice_mut(0).unwrap();

            for z in 0..nz {
                for y in 0..ny {
                    for x in 0..nx {
                        let i = z * nx * ny + y * nx + x;

                        if y < 4 {
                            u[i] = 0.5;
                        }

                        if x > 12 && x < 20 && y > 12 && y < 20 {
                            u[i] += 0.05;
                        }
                    }
                }
            }
        }

        //
        // v component
        //
        {
            let v = velocity.v_mut().time_slice_mut(0).unwrap();

            for z in 0..nz {
                for y in 0..ny {
                    for x in 0..nx {
                        let i = z * nx * ny + y * nx + x;

                        if y < 4 {
                            v[i] = 0.15;
                        }

                        if x > 12 && x < 20 && y > 12 && y < 20 {
                            v[i] -= 0.05;
                        }
                    }
                }
            }
        }

        //
        // w component
        //
        {
            let w = velocity.w_mut().time_slice_mut(0).unwrap();

            for value in w.iter_mut() {
                *value = 0.0;
            }
        }
    }

    let temperature = TemperatureField::new(grid.clone());

    let salinity = SalinityField::new(grid.clone());

    let mut ocean = OceanField::new(coordinates, metadata, velocity, temperature, salinity);

    //
    // Metal solver
    //
    let context = MetalContext::new();

    let solver = OceanSolver::new(context);

    let center = 8 * 32 * 32 + 16 * 32 + 16;

    println!(
        "initial u = {}",
        ocean.velocity().u().time_slice(0).unwrap()[center]
    );

    for step in 0..100 {
        solver.step(&mut ocean, 1.0);

        if step % 10 == 0 {
            let u = ocean.velocity().u().time_slice(0).unwrap()[center];

            println!("step {} : u = {}", step, u);
        }
    }
    let snapshot = gpu_velocity.snapshot(&context);
    println!(
        "velocity size={} center={}",
        snapshot.size(),
        snapshot.u[snapshot.index(8, 8, 8)]
    );
}
