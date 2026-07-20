pub mod buffer;
pub mod camera;
pub mod context;
pub mod depth;
pub mod error;
pub mod frame;
pub mod heightmap;
pub mod mesh;
pub mod pipeline;
pub mod renderer;
pub mod shader;
pub mod surface;
pub mod texture;
pub mod uniform;
// Public renderer API

pub use renderer::Renderer;

// Camera control

pub use camera::Camera;

// GPU mesh

pub use mesh::{GpuMesh, Mesh};

// Surface

pub use surface::RenderSurface;
pub mod test_mesh;

pub use test_mesh::{TestVertex, create_triangle};
pub mod terrain;

pub use heightmap::HeightMap;
pub use terrain::{TerrainMesh, TerrainVertex};
