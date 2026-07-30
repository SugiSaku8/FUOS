use crate::flow_mesh::FlowMesh;

use s_metal::{GpuVelocityField, MetalContext};

pub struct FlowRenderer {}

impl FlowRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build_mesh(
        &self,
        context: &MetalContext,
        velocity: &GpuVelocityField,
        scale: f32,
    ) -> FlowMesh {
        let u = velocity.u.download(context);

        let v = velocity.v.download(context);

        let w = velocity.w.download(context);

        let grid = velocity.grid();

        let mut vertices = Vec::<[f32; 3]>::new();

        //
        // 間引き
        //
        // 全格子を書くと数百万本になるので
        // 4セルごと
        //
        for z in (0..grid.nz()).step_by(4) {
            for y in (0..grid.ny()).step_by(4) {
                for x in (0..grid.nx()).step_by(4) {
                    let index = z as usize * grid.nx() as usize * grid.ny() as usize
                        + y as usize * grid.nx() as usize
                        + x as usize;

                    let start = [x as f32, y as f32, z as f32];

                    let end = [
                        start[0] + u[index] * scale,
                        start[1] + v[index] * scale,
                        start[2] + w[index] * scale,
                    ];

                    vertices.push(start);

                    vertices.push(end);
                }
            }
        }

        FlowMesh::new(context.device(), vertices)
    }
}
