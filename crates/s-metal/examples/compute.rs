use s_metal::{pipeline::create_compute_pipeline, MetalContext};

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

    let _pipeline = create_compute_pipeline(context.device(), &library, "vector_add");

    println!("Compute pipeline created");
}

