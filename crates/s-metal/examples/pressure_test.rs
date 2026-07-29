use s_metal::{GpuField3D, GridInfo, MetalContext, PressureSolver};

fn index(x: usize, y: usize, z: usize, nx: usize, ny: usize) -> usize {
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
    // divergence field
    //

    let mut cpu = vec![0.0f32; grid.count()];

    let cx = nx / 2;
    let cy = ny / 2;
    let cz = nz / 2;

    let center = index(cx, cy, cz, nx, ny);

    cpu[center] = 1.0;

    println!("CPU divergence center = {}", cpu[center]);

    let divergence = GpuField3D::from_f32(&context, &cpu, grid);

    println!(
        "GPU divergence center = {}",
        divergence.download(&context)[center]
    );

    //
    // solver
    //

    let solver = PressureSolver::new(&context);

    let pressure = solver.solve(&context, &divergence, 100);

    let result = pressure.download(&context);

    println!("pressure center = {}", result[center]);

    println!("pressure neighborhood:");

    for z in cz - 1..=cz + 1 {
        for y in cy - 1..=cy + 1 {
            for x in cx - 1..=cx + 1 {
                let i = index(x, y, z, nx, ny);

                println!("{} {} {} : {}", x, y, z, result[i]);
            }
        }
    }
}
