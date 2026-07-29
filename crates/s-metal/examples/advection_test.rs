use s_field::{Grid4D, VelocityField};

use s_metal::{AdvectionOperator, GpuVelocityField, MetalContext};

fn main() {
    //
    // Metal context
    //

    let context = MetalContext::new();

    //
    // Grid
    //
    // x,y,z = 16
    // time = 1
    //

    let nx = 16;
    let ny = 16;
    let nz = 16;

    let grid = Grid4D::new([nx, ny, nz, 1], [1.0, 1.0, 1.0, 1.0]);

    //
    // Velocity field
    //

    let mut velocity = VelocityField::new(grid);

    let mut u = vec![0.0f32; nx * ny * nz];

    let mut v = vec![0.0f32; nx * ny * nz];

    let mut w = vec![0.0f32; nx * ny * nz];

    //
    // Gaussian velocity blob
    //

    let cx = nx as f32 / 2.0;
    let cy = ny as f32 / 2.0;
    let cz = nz as f32 / 2.0;

    let sigma = 3.0;

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let dx = x as f32 - cx;

                let dy = y as f32 - cy;

                let dz = z as f32 - cz;

                let r2 = dx * dx + dy * dy + dz * dz;

                let value = (-r2 / (sigma * sigma)).exp();

                let index = z * nx * ny + y * nx + x;

                //
                // x方向へ流れる速度
                //
                u[index] = value;

                v[index] = 0.0;

                w[index] = 0.0;
            }
        }
    }

    //
    // upload CPU field
    //

    velocity
        .u_mut()
        .time_slice_mut(0)
        .unwrap()
        .copy_from_slice(&u);

    velocity
        .v_mut()
        .time_slice_mut(0)
        .unwrap()
        .copy_from_slice(&v);

    velocity
        .w_mut()
        .time_slice_mut(0)
        .unwrap()
        .copy_from_slice(&w);

    let center = (nz / 2) * nx * ny + (ny / 2) * nx + (nx / 2);

    println!(
        "CPU center velocity = {}",
        velocity.u().time_slice(0).unwrap()[center]
    );

    //
    // GPU upload
    //

    let gpu_velocity = GpuVelocityField::from_velocity(&context, &velocity);

    println!(
        "GPU upload center = {}",
        gpu_velocity.u.download(&context)[center]
    );

    //
    // advection step
    //

    let solver = AdvectionOperator::new(&context);

    let result = solver.compute(&context, &gpu_velocity, 0.1);

    let result_u = result.u.download(&context);

    println!("GPU result center = {}", result_u[center]);

    //
    // 周辺値確認
    //

    println!("center neighborhood:");

    for z in 7..10 {
        for y in 7..10 {
            for x in 7..10 {
                let i = z * nx * ny + y * nx + x;

                println!("{} {} {} : {}", x, y, z, result_u[i]);
            }
        }
    }
}
