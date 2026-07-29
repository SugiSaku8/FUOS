use s_metal::{GridInfo, MetalBuffer, MetalContext};

use s_metal::compute::dispatch;
use s_metal::pipeline::create_compute_pipeline;

fn main() {
    let context = MetalContext::new();

    let shader = include_str!("../shaders/derivative_x.metal");

    let library = context.load_library(shader);

    let pipeline = create_compute_pipeline(context.device(), &library, "derivative_x");

    //
    // f(x)=2x
    //
    let input = MetalBuffer::from_f32(&context, &[0.0, 2.0, 4.0, 6.0]);

    let output = MetalBuffer::empty(&context, 4);

    let grid = GridInfo::new(4, 1, 1, 1.0, 1.0, 1.0);
    let grid_buffer = MetalBuffer::from_struct(&context, &grid);

    dispatch(
        &pipeline,
        context.queue(),
        &[input.raw(), output.raw(), grid_buffer.raw()],
        4,
    );

    println!("{:?}", output.read_f32());
}
