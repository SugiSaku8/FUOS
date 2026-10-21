//! GPU advection operator with an SSP-RK2 option for momentum transport.
use crate::{GpuVelocityField, MetalBuffer, MetalContext};
use metal::MTLSize;

pub struct AdvectionOperator {
    pipeline: metal::ComputePipelineState,
    corrector_pipeline: metal::ComputePipelineState,
}
impl AdvectionOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library=context.load_library(include_str!("../shaders/advection.metal"));
        let function=library.get_function("advection_step",None).expect("advection kernel not found");
        let pipeline=context.device().new_compute_pipeline_state_with_function(&function).expect("failed to create advection pipeline");
        let corrector=library.get_function("advection_ssprk2_correct",None).expect("advection SSP-RK2 corrector not found");
        let corrector_pipeline=context.device().new_compute_pipeline_state_with_function(&corrector).expect("failed to create advection corrector pipeline");
        Self{pipeline,corrector_pipeline}
    }

    /// First-order Euler advection, retained for compatibility and kernel diagnostics.
    pub fn compute(&self, context:&MetalContext, velocity:&GpuVelocityField, dt:f32)->GpuVelocityField {
        let grid=velocity.grid(); let output=GpuVelocityField::empty(context,grid);
        let grid_buffer=MetalBuffer::from_struct(context,&grid); let dt_buffer=MetalBuffer::from_f32(context.device(),&[dt]);
        let command_buffer=context.command_queue().new_command_buffer(); let encoder=command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline);
        encoder.set_buffer(0,Some(velocity.u.buffer()),0);encoder.set_buffer(1,Some(velocity.v.buffer()),0);encoder.set_buffer(2,Some(velocity.w.buffer()),0);
        encoder.set_buffer(3,Some(output.u.buffer()),0);encoder.set_buffer(4,Some(output.v.buffer()),0);encoder.set_buffer(5,Some(output.w.buffer()),0);
        encoder.set_buffer(6,Some(grid_buffer.buffer()),0);encoder.set_buffer(7,Some(dt_buffer.buffer()),0);
        let threads=MTLSize{width:8,height:8,depth:8};let groups=groups_for(grid);
        encoder.dispatch_thread_groups(groups,threads);encoder.end_encoding();command_buffer.commit();command_buffer.wait_until_completed();output
    }

    /// Second-order SSP Runge-Kutta for the nonlinear advection term. Both stages are
    /// encoded in one command buffer, so no CPU/GPU synchronization occurs between them.
    pub fn compute_ssprk2(&self, context:&MetalContext, velocity:&GpuVelocityField, dt:f32)->GpuVelocityField {
        let grid=velocity.grid(); let stage=GpuVelocityField::empty(context,grid); let output=GpuVelocityField::empty(context,grid);
        let grid_buffer=MetalBuffer::from_struct(context,&grid); let dt_buffer=MetalBuffer::from_f32(context.device(),&[dt]);
        let command_buffer=context.command_queue().new_command_buffer();
        let first=command_buffer.new_compute_command_encoder();first.set_compute_pipeline_state(&self.pipeline);
        first.set_buffer(0,Some(velocity.u.buffer()),0);first.set_buffer(1,Some(velocity.v.buffer()),0);first.set_buffer(2,Some(velocity.w.buffer()),0);
        first.set_buffer(3,Some(stage.u.buffer()),0);first.set_buffer(4,Some(stage.v.buffer()),0);first.set_buffer(5,Some(stage.w.buffer()),0);
        first.set_buffer(6,Some(grid_buffer.buffer()),0);first.set_buffer(7,Some(dt_buffer.buffer()),0);
        let threads=MTLSize{width:8,height:8,depth:8};let groups=groups_for(grid);
        first.dispatch_thread_groups(groups,threads);first.end_encoding();
        let second=command_buffer.new_compute_command_encoder();second.set_compute_pipeline_state(&self.corrector_pipeline);
        second.set_buffer(0,Some(velocity.u.buffer()),0);second.set_buffer(1,Some(velocity.v.buffer()),0);second.set_buffer(2,Some(velocity.w.buffer()),0);
        second.set_buffer(3,Some(stage.u.buffer()),0);second.set_buffer(4,Some(stage.v.buffer()),0);second.set_buffer(5,Some(stage.w.buffer()),0);
        second.set_buffer(6,Some(output.u.buffer()),0);second.set_buffer(7,Some(output.v.buffer()),0);second.set_buffer(8,Some(output.w.buffer()),0);
        second.set_buffer(9,Some(grid_buffer.buffer()),0);second.set_buffer(10,Some(dt_buffer.buffer()),0);
        second.dispatch_thread_groups(groups,threads);second.end_encoding();command_buffer.commit();command_buffer.wait_until_completed();output
    }
}

fn groups_for(grid: crate::GridInfo)->MTLSize {
    MTLSize{width:((grid.nx()+7)/8) as u64,height:((grid.ny()+7)/8) as u64,depth:((grid.nz()+7)/8) as u64}
}
