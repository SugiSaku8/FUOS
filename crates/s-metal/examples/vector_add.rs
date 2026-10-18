use s_metal::{MetalBuffer, MetalContext};

use s_metal::compute::dispatch;
use s_metal::pipeline::create_compute_pipeline;

fn main() {
    let context = MetalContext::new();

    let shader = r#"
#include <metal_stdlib>

using namespace metal;


kernel void vector_add(

device const float* a [[buffer(0)]],

device const float* b [[buffer(1)]],

device float* c [[buffer(2)]],

uint id [[thread_position_in_grid]]

)

{

    c[id]=a[id]+b[id];

}

"#;

    let library = context.load_library(shader);

    let pipeline = create_compute_pipeline(context.device(), &library, "vector_add");

    let a = MetalBuffer::from_f32(&context, &[1.0, 2.0, 3.0, 4.0]);

    let b = MetalBuffer::from_f32(&context, &[10.0, 20.0, 30.0, 40.0]);

    let result = MetalBuffer::empty(&context, 4);

    dispatch(
        &pipeline,
        context.queue(),
        &[a.raw(), b.raw(), result.raw()],
        4,
    );

    println!("{:?}", result.read_f32());
}

