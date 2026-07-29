use s_metal::MetalContext;

fn main() {
    let metal = MetalContext::new();

    println!("Metal device initialized");

    println!("{:?}", metal.device().name());
}
