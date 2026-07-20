use nalgebra::Vector3;

use crate::mesh::Mesh;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainVertex {
    pub position: [f32; 3],

    pub normal: [f32; 3],

    pub height: f32,
}

impl TerrainVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TerrainVertex>() as wgpu::BufferAddress,

            step_mode: wgpu::VertexStepMode::Vertex,

            attributes: &ATTRIBUTES,
        }
    }
}

pub struct TerrainMesh {
    pub vertices: Vec<TerrainVertex>,

    pub indices: Vec<u32>,
}

impl TerrainMesh {
    //
    // テスト用地形
    //
    pub fn new(size: usize) -> Self {
        let mut vertices = Vec::new();

        let mut indices = Vec::new();

        for z in 0..=size {
            for x in 0..=size {
                let xf = x as f32 / size as f32;

                let zf = z as f32 / size as f32;

                let height = ((xf * 12.0).sin() * (zf * 12.0).cos() * 5.0) - 10.0;

                vertices.push(TerrainVertex {
                    position: [x as f32, height, z as f32],

                    normal: [0.0, 1.0, 0.0],

                    height,
                });
            }
        }

        for z in 0..size {
            for x in 0..size {
                let i = z * (size + 1) + x;

                indices.extend_from_slice(&[
                    i as u32,
                    (i + size + 1) as u32,
                    (i + 1) as u32,
                    (i + 1) as u32,
                    (i + size + 1) as u32,
                    (i + size + 2) as u32,
                ]);
            }
        }

        Self::calculate_normals(&mut vertices, &indices);

        Self { vertices, indices }
    }

    //
    // HeightMapから生成
    //
    pub fn from_heightmap(heightmap: &crate::heightmap::HeightMap, scale: f32) -> Self {
        let width = heightmap.width;

        let height = heightmap.height;

        let mut vertices = Vec::new();

        let mut indices = Vec::new();

        //
        // 頂点生成
        //
        for z in 0..height {
            for x in 0..width {
                let h = heightmap.get(x, z) * scale;

                vertices.push(TerrainVertex {
                    position: [x as f32, h, z as f32],

                    normal: [0.0, 1.0, 0.0],

                    height: h,
                });
            }
        }

        //
        // 面生成
        //
        for z in 0..height - 1 {
            for x in 0..width - 1 {
                let i = z * width + x;

                indices.extend_from_slice(&[
                    i as u32,
                    (i + width) as u32,
                    (i + 1) as u32,
                    (i + 1) as u32,
                    (i + width) as u32,
                    (i + width + 1) as u32,
                ]);
            }
        }

        Self::calculate_normals(&mut vertices, &indices);

        Self { vertices, indices }
    }

    //
    // GPUアップロード用
    //
    pub fn mesh(&self) -> Mesh<TerrainVertex> {
        Mesh {
            vertices: self.vertices.clone(),

            indices: self.indices.clone(),
        }
    }

    //
    // 法線計算
    //
    fn calculate_normals(vertices: &mut Vec<TerrainVertex>, indices: &[u32]) {
        for vertex in vertices.iter_mut() {
            vertex.normal = [0.0, 0.0, 0.0];
        }

        for triangle in indices.chunks(3) {
            let i0 = triangle[0] as usize;

            let i1 = triangle[1] as usize;

            let i2 = triangle[2] as usize;

            let p0 = Vector3::from(vertices[i0].position);

            let p1 = Vector3::from(vertices[i1].position);

            let p2 = Vector3::from(vertices[i2].position);

            let edge1 = p1 - p0;

            let edge2 = p2 - p0;

            let normal = edge1.cross(&edge2).normalize();

            for i in [i0, i1, i2] {
                vertices[i].normal[0] += normal.x;

                vertices[i].normal[1] += normal.y;

                vertices[i].normal[2] += normal.z;
            }
        }

        for vertex in vertices.iter_mut() {
            let normal =
                Vector3::new(vertex.normal[0], vertex.normal[1], vertex.normal[2]).normalize();

            vertex.normal = [normal.x, normal.y, normal.z];
        }
    }

    pub fn upload(&self, device: &wgpu::Device) -> crate::mesh::GpuMesh<TerrainVertex> {
        let mesh = Mesh {
            vertices: self.vertices.clone(),

            indices: self.indices.clone(),
        };

        crate::mesh::GpuMesh::upload(device, &mesh)
    }
}
