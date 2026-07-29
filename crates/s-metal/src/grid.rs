#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GridInfo {
    pub nx: u32,

    pub ny: u32,

    pub nz: u32,

    pub dx: f32,

    pub dy: f32,

    pub dz: f32,
}

impl GridInfo {
    pub fn new(nx: u32, ny: u32, nz: u32, dx: f32, dy: f32, dz: f32) -> Self {
        Self {
            nx,

            ny,

            nz,

            dx,

            dy,

            dz,
        }
    }

    pub fn count(&self) -> usize {
        (self.nx * self.ny * self.nz) as usize
    }
}
