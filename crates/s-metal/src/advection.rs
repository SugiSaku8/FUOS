use crate::{GpuField3D, GpuVelocityField, MetalBuffer, MetalContext, VelocityGradient};

use crate::compute::dispatch_custom;

use metal::ComputePipelineState;

/// GPU Advection operator.
///
/// Computes:
///
/// -(u · ∇)u
///
/// for three-dimensional ocean velocity fields.
pub struct AdvectionOperator {
    pipeline: ComputePipelineState,
}

impl AdvectionOperator {
    pub fn new(pipeline: ComputePipelineState) -> Self {
        Self { pipeline }
    }

    /// Compute advection term.
    ///
    /// Returns:
    ///
    /// du/dt = -(u · ∇)u
    ///
    pub fn compute(
        &self,

        context: &MetalContext,

        velocity: &GpuVelocityField,

        gradient: &VelocityGradient,
    ) -> GpuField3D {
        let grid = velocity.u.grid();

        let output = GpuField3D::empty(context, grid);

        let grid_buffer = MetalBuffer::from_struct(context, &grid);

        dispatch_custom(
            &self.pipeline,
            context.queue(),
            &[
                // velocity
                velocity.u.buffer(),
                velocity.v.buffer(),
                velocity.w.buffer(),
                // gradients of u
                gradient.u_dx.buffer(),
                gradient.u_dy.buffer(),
                gradient.u_dz.buffer(),
                // output
                output.buffer(),
                // grid information
                grid_buffer.raw(),
            ],
            grid.count(),
        );

        output
    }
}
