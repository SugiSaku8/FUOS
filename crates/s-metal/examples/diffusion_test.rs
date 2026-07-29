use s_metal::{DiffusionOperator, GpuField3D, GridInfo, MetalContext};

fn main() {
    let context = MetalContext::new();

    //
    // grid
    //
    let nx = 16usize;
    let ny = 16usize;
    let nz = 16usize;

    let grid = GridInfo::new(nx as u32, ny as u32, nz as u32, 1.0, 1.0, 1.0);

    //
    // CPU field
    //
    let mut cpu = vec![0.0f32; grid.count()];

    let cx = nx / 2;
    let cy = ny / 2;
    let cz = nz / 2;

    let center = cz * nx * ny + cy * nx + cx;

    cpu[center] = 1.0;

    println!("CPU center = {}", cpu[center]);

    //
    // upload
    //
    let field = GpuField3D::from_f32(&context, &cpu, grid);

    println!("GPU upload center = {}", field.download(&context)[center]);

    //
    // diffusion
    //
    let diffusion = DiffusionOperator::new(&context);

    let output = diffusion.compute(
        &context, &field, 0.1,  // viscosity
        0.01, // dt
    );

    let result = output.download(&context);

    println!("GPU result center = {}", result[center]);

    println!("center neighborhood:");

    for z in (cz - 1)..=(cz + 1) {
        for y in (cy - 1)..=(cy + 1) {
            for x in (cx - 1)..=(cx + 1) {
                let i = z * nx * ny + y * nx + x;

                println!("{} {} {} : {}", x, y, z, result[i]);
            }
        }
    }
}
