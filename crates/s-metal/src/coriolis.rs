//! Latitude-dependent Coriolis rotation.
use crate::{GpuVelocityField, MetalBuffer, MetalContext};
use metal::MTLSize;

pub const EARTH_ANGULAR_VELOCITY_RAD_S: f64 = 7.292_115_0e-5;

pub fn coriolis_parameter(latitude_degrees: f64) -> f32 {
    (2.0 * EARTH_ANGULAR_VELOCITY_RAD_S * latitude_degrees.to_radians().sin()) as f32
}

pub struct CoriolisOperator { pipeline: metal::ComputePipelineState }
impl CoriolisOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library=context.load_library(include_str!("../shaders/coriolis.metal"));
        let function=library.get_function("coriolis_step",None).expect("Coriolis kernel not found");
        let pipeline=context.device().new_compute_pipeline_state_with_function(&function).expect("failed to create Coriolis pipeline");
        Self{pipeline}
    }
    pub fn compute(&self, context:&MetalContext, velocity:&GpuVelocityField, f_by_latitude:&[f32], dt:f32)->GpuVelocityField {
        let grid=velocity.grid();
        let f=if f_by_latitude.len()==grid.ny() as usize {f_by_latitude.to_vec()} else {vec![0.0;grid.ny() as usize]};
        let f_buffer=MetalBuffer::from_f32(context.device(),&f);
        let grid_buffer=MetalBuffer::from_struct(context,&grid);
        let dt_buffer=MetalBuffer::from_f32(context.device(),&[dt]);
        let output=GpuVelocityField::empty(context,grid);
        let command_buffer=context.command_queue().new_command_buffer();
        let encoder=command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline);
        encoder.set_buffer(0,Some(velocity.u.buffer()),0); encoder.set_buffer(1,Some(velocity.v.buffer()),0);
        encoder.set_buffer(2,Some(velocity.w.buffer()),0); encoder.set_buffer(3,Some(output.u.buffer()),0);
        encoder.set_buffer(4,Some(output.v.buffer()),0); encoder.set_buffer(5,Some(output.w.buffer()),0);
        encoder.set_buffer(6,Some(f_buffer.buffer()),0); encoder.set_buffer(7,Some(grid_buffer.buffer()),0);
        encoder.set_buffer(8,Some(dt_buffer.buffer()),0);
        let threads=MTLSize{width:8,height:8,depth:8};
        let groups=MTLSize{width:((grid.nx()+7)/8) as u64,height:((grid.ny()+7)/8) as u64,depth:((grid.nz()+7)/8) as u64};
        encoder.dispatch_thread_groups(groups,threads); encoder.end_encoding(); command_buffer.commit(); command_buffer.wait_until_completed();
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coriolis_parameter_has_expected_sign_and_magnitude() {
        let north=coriolis_parameter(35.0);
        let south=coriolis_parameter(-35.0);
        assert!(north>0.0 && south<0.0);
        assert!((north-8.36e-5).abs()<3.0e-7);
    }
}
