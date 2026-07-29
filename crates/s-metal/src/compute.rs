use metal::{Buffer, CommandQueue, ComputePipelineState, MTLSize};

pub fn dispatch(
    pipeline: &ComputePipelineState,
    queue: &CommandQueue,
    buffers: &[&Buffer],
    count: usize,
) {
    let command_buffer = queue.new_command_buffer();

    let encoder = command_buffer.new_compute_command_encoder();

    encoder.set_compute_pipeline_state(pipeline);

    for (index, buffer) in buffers.iter().enumerate() {
        encoder.set_buffer(index as u64, Some(buffer), 0);
    }

    // Metal thread width
    // u64 -> usize
    let width = pipeline.thread_execution_width() as usize;

    let threads = MTLSize {
        width: width as u64,
        height: 1,
        depth: 1,
    };

    let groups = MTLSize {
        width: ((count + width - 1) / width) as u64,

        height: 1,
        depth: 1,
    };

    encoder.dispatch_thread_groups(groups, threads);

    encoder.end_encoding();

    command_buffer.commit();

    command_buffer.wait_until_completed();
}

pub fn dispatch_with_grid(
    pipeline: &ComputePipelineState,
    queue: &CommandQueue,

    input: &Buffer,
    output: &Buffer,

    grid_buffer: &Buffer,

    count: usize,
) {
    let command_buffer = queue.new_command_buffer();

    let encoder = command_buffer.new_compute_command_encoder();

    encoder.set_compute_pipeline_state(pipeline);

    encoder.set_buffer(0, Some(input), 0);

    encoder.set_buffer(1, Some(output), 0);

    encoder.set_buffer(2, Some(grid_buffer), 0);

    let width = pipeline.thread_execution_width() as usize;

    let threads = MTLSize {
        width: width as u64,

        height: 1,

        depth: 1,
    };

    let groups = MTLSize {
        width: ((count + width - 1) / width) as u64,

        height: 1,

        depth: 1,
    };

    encoder.dispatch_thread_groups(groups, threads);

    encoder.end_encoding();

    command_buffer.commit();

    command_buffer.wait_until_completed();
}

pub fn dispatch_custom(
    pipeline: &ComputePipelineState,
    queue: &CommandQueue,

    buffers: &[&Buffer],

    count: usize,
) {
    let command_buffer = queue.new_command_buffer();

    let encoder = command_buffer.new_compute_command_encoder();

    encoder.set_compute_pipeline_state(pipeline);

    for (index, buffer) in buffers.iter().enumerate() {
        encoder.set_buffer(index as u64, Some(buffer), 0);
    }

    let width = pipeline.thread_execution_width() as usize;

    let threads = MTLSize {
        width: width as u64,

        height: 1,

        depth: 1,
    };

    let groups = MTLSize {
        width: ((count + width - 1) / width) as u64,

        height: 1,

        depth: 1,
    };

    encoder.dispatch_thread_groups(groups, threads);

    encoder.end_encoding();

    command_buffer.commit();

    command_buffer.wait_until_completed();
}
