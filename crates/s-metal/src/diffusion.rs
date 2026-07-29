//! GPU diffusion operator.

use crate::{GpuField3D, MetalBuffer, MetalContext};

use metal::MTLSize;

pub struct DiffusionOperator {
    pipeline: metal::ComputePipelineState,
}

impl DiffusionOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/diffusion.metal"));

        let function = library
            .get_function("diffusion_step", None)
            .expect("diffusion kernel not found");

        let pipeline = context
            .device()
            .new_compute_pipeline_state_with_function(&function)
            .expect("failed to create diffusion pipeline");

        Self { pipeline }
    }

    pub fn compute(
        &self,
        context: &MetalContext,
        field: &GpuField3D,
        viscosity: f32,
        dt: f32,
    ) -> GpuField3D {
        let grid = field.grid();

        let output = GpuField3D::empty(context, grid);

        let command_buffer = context.command_queue().new_command_buffer();

        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.pipeline);

        encoder.set_buffer(0, Some(field.buffer()), 0);

        encoder.set_buffer(1, Some(output.buffer()), 0);

        let grid_buffer = MetalBuffer::from_struct(context, &grid);

        encoder.set_buffer(2, Some(grid_buffer.raw()), 0);

        let viscosity_buffer = MetalBuffer::from_f32(context.device(), &[viscosity]);

        encoder.set_buffer(3, Some(viscosity_buffer.raw()), 0);

        let dt_buffer = MetalBuffer::from_f32(context.device(), &[dt]);

        encoder.set_buffer(4, Some(dt_buffer.raw()), 0);

        let threads = MTLSize {
            width: 8,
            height: 8,
            depth: 8,
        };

        let groups = MTLSize {
            width: ((grid.nx() + 7) / 8) as u64,

            height: ((grid.ny() + 7) / 8) as u64,

            depth: ((grid.nz() + 7) / 8) as u64,
        };

        encoder.dispatch_thread_groups(groups, threads);

        encoder.end_encoding();

        command_buffer.commit();

        command_buffer.wait_until_completed();

        output
    }
}
