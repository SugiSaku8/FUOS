//! GPU diffusion operator.

use crate::{GpuField3D, GpuVelocityField, MetalBuffer, MetalContext};

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
        &self, context: &MetalContext, field: &GpuField3D, viscosity: f32, dt: f32,
    ) -> GpuField3D {
        self.compute_anisotropic(context, field, viscosity, viscosity, dt)
    }

    pub fn compute_anisotropic(
        &self, context: &MetalContext, field: &GpuField3D,
        viscosity_horizontal: f32, viscosity_vertical: f32, dt: f32,
    ) -> GpuField3D {
        let grid = field.grid();
        let output = GpuField3D::empty(context, grid);
        let command_buffer = context.command_queue().new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline);
        encoder.set_buffer(0, Some(field.buffer()), 0);
        encoder.set_buffer(1, Some(output.buffer()), 0);
        let grid_buffer = MetalBuffer::from_struct(context, &grid);
        encoder.set_buffer(2, Some(grid_buffer.buffer()), 0);
        let horizontal_buffer = MetalBuffer::from_f32(context.device(), &[viscosity_horizontal.max(0.0)]);
        let vertical_buffer = MetalBuffer::from_f32(context.device(), &[viscosity_vertical.max(0.0)]);
        let dt_buffer = MetalBuffer::from_f32(context.device(), &[dt]);
        encoder.set_buffer(3, Some(horizontal_buffer.buffer()), 0);
        encoder.set_buffer(4, Some(vertical_buffer.buffer()), 0);
        encoder.set_buffer(5, Some(dt_buffer.buffer()), 0);
        let threads = MTLSize { width: 8, height: 8, depth: 8 };
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

    pub fn compute_velocity(
        &self, context: &MetalContext, velocity: &GpuVelocityField, viscosity: f32, dt: f32,
    ) -> GpuVelocityField {
        self.compute_velocity_anisotropic(context, velocity, viscosity, viscosity, dt)
    }

    pub fn compute_velocity_anisotropic(
        &self, context: &MetalContext, velocity: &GpuVelocityField,
        viscosity_horizontal: f32, viscosity_vertical: f32, dt: f32,
    ) -> GpuVelocityField {
        let u = self.compute_anisotropic(context, &velocity.u, viscosity_horizontal, viscosity_vertical, dt);
        let v = self.compute_anisotropic(context, &velocity.v, viscosity_horizontal, viscosity_vertical, dt);
        let w = self.compute_anisotropic(context, &velocity.w, viscosity_horizontal, viscosity_vertical, dt);
        GpuVelocityField { u, v, w, grid: velocity.grid() }
    }

}
