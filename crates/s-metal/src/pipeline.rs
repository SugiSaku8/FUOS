use metal::{ComputePipelineDescriptor, ComputePipelineState, Device, Library};

pub fn create_compute_pipeline(
    device: &Device,
    library: &Library,
    name: &str,
) -> ComputePipelineState {
    let function = library.get_function(name, None).unwrap();

    let descriptor = ComputePipelineDescriptor::new();

    descriptor.set_compute_function(Some(&function));

    device.new_compute_pipeline_state(&descriptor).unwrap()
}
