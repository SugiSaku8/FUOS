use s_metal::{GpuField3D, GpuVelocityField, GradientPressureOperator, GridInfo, MetalContext};

fn index3d(x: usize, y: usize, z: usize, nx: usize, ny: usize) -> usize {
    z * nx * ny + y * nx + x
}

fn main() {
    let context = MetalContext::new();

    //
    // grid
    //
    let nx = 16;
    let ny = 16;
    let nz = 16;

    let grid = GridInfo::new(nx as u32, ny as u32, nz as u32, 1.0, 1.0, 1.0);

    //
    // pressure field
    //
    let mut pressure = vec![0.0f32; grid.count()];

    let cx = nx / 2;
    let cy = ny / 2;
    let cz = nz / 2;

    let center = index3d(cx, cy, cz, nx, ny);

    pressure[center] = 1.0;

    println!("CPU pressure center = {}", pressure[center]);

    let gpu_pressure = GpuField3D::from_f32(&context, &pressure, grid);

    println!(
        "GPU pressure center = {}",
        gpu_pressure.download(&context)[center]
    );

    //
    // velocity field
    //
    // zero velocity
    //
    let gpu_velocity = GpuVelocityField::empty(&context, grid);

    println!(
        "GPU upload center = {}",
        gpu_velocity.u.download(&context)[center]
    );

    //
    // pressure gradient correction
    //
    let operator = GradientPressureOperator::new(&context);

    let corrected = operator.compute(&context, &gpu_velocity, &gpu_pressure, 0.01, 1025.0);

    let u = corrected.u.download(&context);

    let v = corrected.v.download(&context);

    let w = corrected.w.download(&context);

    println!("velocity correction center:");

    println!("u = {}", u[center]);

    println!("v = {}", v[center]);

    println!("w = {}", w[center]);

    println!("neighborhood:");

    for z in cz - 1..=cz + 1 {
        for y in cy - 1..=cy + 1 {
            for x in cx - 1..=cx + 1 {
                let i = index3d(x, y, z, nx, ny);

                println!("{} {} {} : ({}, {}, {})", x, y, z, u[i], v[i], w[i],);
            }
        }
    }
}
