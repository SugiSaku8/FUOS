use crate::{grid::OceanGrid, mesh::TerrainMesh, vertex::TerrainVertex};

pub struct TerrainBuilder;

impl TerrainBuilder {
    pub fn build(grid: &OceanGrid) -> TerrainMesh {
        let mut mesh = TerrainMesh::new();

        let width = grid.width();
        let height = grid.height();

        for y in 0..height {
            for x in 0..width {
                let depth = grid.depth(x, y);

                mesh.vertices.push(TerrainVertex {
                    position: [x as f32, y as f32, -depth],

                    normal: [0.0, 0.0, 1.0],
                });
            }
        }

        for y in 0..height - 1 {
            for x in 0..width - 1 {
                let i = (y * width + x) as u32;

                mesh.indices.extend_from_slice(&[
                    i,
                    i + 1,
                    i + width as u32,
                    i + 1,
                    i + width as u32 + 1,
                    i + width as u32,
                ]);
            }
        }

        mesh
    }
}
