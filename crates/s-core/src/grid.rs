#[derive(Clone)]
pub struct GridSize {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
}

impl GridSize {
    pub fn new(nx: usize, ny: usize, nz: usize) -> Self {
        Self { nx, ny, nz }
    }
}
