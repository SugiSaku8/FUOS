//! Density-anomaly buoyancy forcing on Metal.
use crate::{GpuVelocityField, MetalBuffer, MetalContext};
use metal::MTLSize;

pub struct BuoyancyOperator { pipeline: metal::ComputePipelineState }
impl BuoyancyOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/buoyancy.metal"));
        let function = library.get_function("buoyancy_step", None).expect("buoyancy kernel not found");
        let pipeline = context.device().new_compute_pipeline_state_with_function(&function)
            .expect("failed to create buoyancy pipeline");
        Self { pipeline }
    }

    pub fn compute_density(
        &self, context: &MetalContext, velocity: &GpuVelocityField,
        density: &[f32], reference_density_by_depth: &[f32], dt: f32,
    ) -> GpuVelocityField {
        let grid = velocity.grid();
        let n = grid.count();
        assert_eq!(density.len(), n, "density field must match velocity grid");
        assert_eq!(reference_density_by_depth.len(), grid.nz() as usize,
            "reference density must have one value per depth layer");
        let density_buffer = MetalBuffer::from_f32(context.device(), density);
        let reference_buffer = MetalBuffer::from_f32(context.device(), reference_density_by_depth);
        let output = GpuVelocityField::empty(context, grid);
        let grid_buffer = MetalBuffer::from_struct(context, &grid);
        let dt_buffer = MetalBuffer::from_f32(context.device(), &[dt]);
        let command_buffer = context.command_queue().new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline);
        encoder.set_buffer(0, Some(density_buffer.buffer()), 0);
        encoder.set_buffer(1, Some(reference_buffer.buffer()), 0);
        encoder.set_buffer(2, Some(velocity.u.buffer()), 0);
        encoder.set_buffer(3, Some(velocity.v.buffer()), 0);
        encoder.set_buffer(4, Some(velocity.w.buffer()), 0);
        encoder.set_buffer(5, Some(output.u.buffer()), 0);
        encoder.set_buffer(6, Some(output.v.buffer()), 0);
        encoder.set_buffer(7, Some(output.w.buffer()), 0);
        encoder.set_buffer(8, Some(grid_buffer.buffer()), 0);
        encoder.set_buffer(9, Some(dt_buffer.buffer()), 0);
        let threads = MTLSize { width: 8, height: 8, depth: 8 };
        let groups = MTLSize {
            width: ((grid.nx()+7)/8) as u64,
            height: ((grid.ny()+7)/8) as u64,
            depth: ((grid.nz()+7)/8) as u64,
        };
        encoder.dispatch_thread_groups(groups, threads);
        encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();
        output
    }
}
