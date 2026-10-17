use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DatasetMetadata {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub nt: usize,

    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
    pub dt: f32,

    pub origin_lat: f32,
    pub origin_lon: f32,
}

impl DatasetMetadata {
    pub fn grid_size(&self) -> [usize; 4] {
        [self.nx, self.ny, self.nz, self.nt]
    }

    pub fn spacing(&self) -> [f32; 4] {
        [self.dx, self.dy, self.dz, self.dt]
    }
}
