use s_field::{Grid4D, VelocityField};
use s_metal::{DivergenceOperator, GpuVelocityField, MetalContext};

fn main() {
    let context = MetalContext::new();

    let nx = 16;
    let ny = 16;
    let nz = 16;

    let grid = Grid4D::new([1, nz, ny, nx], [1.0, 1.0, 1.0, 1.0]);

    let mut velocity = VelocityField::new(grid);

    let size = nx * ny * nz;

    let mut u = vec![0.0; size];

    let v = vec![0.0; size];

    let w = vec![0.0; size];

    let cx = nx / 2;
    let cy = ny / 2;
    let cz = nz / 2;

    let index = cz * nx * ny + cy * nx + cx;

    u[index] = 1.0;

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

    println!("CPU center velocity = {}", u[index]);

    let gpu_velocity = GpuVelocityField::from_velocity(&context, &velocity);

    println!(
        "GPU upload center = {}",
        gpu_velocity.u.download(&context)[index]
    );

    let operator = DivergenceOperator::new(&context);

    let divergence = operator.compute(&context, &gpu_velocity);

    let result = divergence.download(&context);

    println!("divergence center = {}", result[index]);

    println!("neighborhood:");

    for z in cz - 1..=cz + 1 {
        for y in cy - 1..=cy + 1 {
            for x in cx - 1..=cx + 1 {
                let i = z * nx * ny + y * nx + x;

                println!("{} {} {} : {}", x, y, z, result[i]);
            }
        }
    }
}
