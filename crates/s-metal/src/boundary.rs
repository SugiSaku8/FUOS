use crate::{GpuVelocityField, MetalBuffer, MetalContext};
use metal::MTLSize;
use s_field::{Grid4D, VelocityField};

/// GPU-applied velocity boundary. The simulator constructor should capture the
/// eastern-face values from the supplied ocean dataset instead of inventing a flow.
pub struct BoundaryOperator {
    pipeline: metal::ComputePipelineState,
    prescribed: Option<(MetalBuffer, MetalBuffer, MetalBuffer, [u32; 3])>,
    dummy: (MetalBuffer, MetalBuffer, MetalBuffer),
}

impl BoundaryOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/boundary.metal"));
        let function = library.get_function("boundary_step", None).expect("boundary kernel not found");
        let pipeline = context.device().new_compute_pipeline_state_with_function(&function)
            .expect("failed to create boundary pipeline");
        let dummy = (
            MetalBuffer::from_f32(context.device(), &[0.0]),
            MetalBuffer::from_f32(context.device(), &[0.0]),
            MetalBuffer::from_f32(context.device(), &[0.0]),
        );
        Self { pipeline, prescribed: None, dummy }
    }

    /// Capture the source dataset's eastern boundary once. The captured face remains
    /// fixed through the run; time-dependent open-boundary forcing is a later step.
    pub fn from_velocity(context: &MetalContext, velocity: &VelocityField) -> Self {
        let mut operator = Self::new(context);
        let shape = velocity.shape(); // [depth, latitude, longitude]
        let nz = shape[0];
        let ny = shape[1];
        let nx = shape[2];
        let plane = nx * ny;
        let u = velocity.u().time_slice(0).unwrap_or(&[]);
        let v = velocity.v().time_slice(0).unwrap_or(&[]);
        let w = velocity.w().time_slice(0).unwrap_or(&[]);
        if nx > 0 && ny > 0 && nz > 0 && u.len() == plane * nz && v.len() == u.len() && w.len() == u.len() {
            let mut face_u = Vec::with_capacity(ny * nz);
            let mut face_v = Vec::with_capacity(ny * nz);
            let mut face_w = Vec::with_capacity(ny * nz);
            for z in 0..nz {
                for y in 0..ny {
                    let i = z * plane + y * nx + nx - 1;
                    face_u.push(u[i]); face_v.push(v[i]); face_w.push(w[i]);
                }
            }
            operator.prescribed = Some((
                MetalBuffer::from_f32(context.device(), &face_u),
                MetalBuffer::from_f32(context.device(), &face_v),
                MetalBuffer::from_f32(context.device(), &face_w),
                [nx as u32, ny as u32, nz as u32],
            ));
        }
        operator
    }

    /// Retained for tests/examples which need the legacy synthetic inflow field.
    pub fn kuroshio_inflow(&self, context: &MetalContext, velocity: &GpuVelocityField) -> GpuVelocityField {
        let grid_info = velocity.grid();
        let nx = grid_info.nx() as usize; let ny = grid_info.ny() as usize; let nz = grid_info.nz() as usize;
        let grid = Grid4D::new([1, nz, ny, nx], [1.0, grid_info.dz(), grid_info.dy(), grid_info.dx()]);
        let mut field = VelocityField::new(grid);
        let mut u = vec![0.0_f32; nx*ny*nz]; let mut v = vec![0.0_f32; nx*ny*nz];
        let w = vec![0.0_f32; nx*ny*nz];
        for z in 0..nz { let f = if nz>1 { 1.0-z as f32/(nz-1) as f32 } else { 1.0 };
            for y in 0..ny { let i=z*nx*ny+y*nx+nx.saturating_sub(1); if i<u.len(){u[i]=0.8*f;v[i]=0.15*f;} }
        }
        field.u_mut().replace(u); field.v_mut().replace(v); field.w_mut().replace(w);
        GpuVelocityField::from_velocity(context, &field)
    }

    /// Apply the prescribed face entirely on the GPU (no velocity-field CPU round trip).
    pub fn apply_velocity(&self, context: &MetalContext, velocity: &mut GpuVelocityField) {
        let grid = velocity.grid();
        if grid.nx() == 0 || grid.ny() == 0 || grid.nz() == 0 { return; }
        let output = GpuVelocityField::empty(context, grid);
        let grid_buffer = MetalBuffer::from_struct(context, &grid);
        let mode_value: u32;
        let (face_u, face_v, face_w) = if let Some((u,v,w,dims)) = &self.prescribed {
            if *dims == [grid.nx(),grid.ny(),grid.nz()] { mode_value=1; (u.buffer(),v.buffer(),w.buffer()) }
            else { mode_value=0; (self.dummy.0.buffer(),self.dummy.1.buffer(),self.dummy.2.buffer()) }
        } else { mode_value=0; (self.dummy.0.buffer(),self.dummy.1.buffer(),self.dummy.2.buffer()) };
        let mode_buffer = MetalBuffer::from_struct(context, &mode_value);
        let command_buffer = context.command_queue().new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline);
        encoder.set_buffer(0, Some(velocity.u.buffer()), 0); encoder.set_buffer(1, Some(velocity.v.buffer()), 0);
        encoder.set_buffer(2, Some(velocity.w.buffer()), 0); encoder.set_buffer(3, Some(output.u.buffer()), 0);
        encoder.set_buffer(4, Some(output.v.buffer()), 0); encoder.set_buffer(5, Some(output.w.buffer()), 0);
        encoder.set_buffer(6, Some(grid_buffer.buffer()), 0); encoder.set_buffer(7, Some(face_u), 0);
        encoder.set_buffer(8, Some(face_v), 0); encoder.set_buffer(9, Some(face_w), 0);
        encoder.set_buffer(10, Some(mode_buffer.buffer()), 0);
        let threads=MTLSize{width:8,height:8,depth:8};
        let groups=MTLSize{width:((grid.nx()+7)/8) as u64,height:((grid.ny()+7)/8) as u64,depth:((grid.nz()+7)/8) as u64};
        encoder.dispatch_thread_groups(groups,threads); encoder.end_encoding(); command_buffer.commit(); command_buffer.wait_until_completed();
        *velocity=output;
    }
}
