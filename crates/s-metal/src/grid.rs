//! GPU grid information.

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GridInfo {
    nx: u32,
    ny: u32,
    nz: u32,

    dx: f32,
    dy: f32,
    dz: f32,
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

    #[inline]
    pub fn nx(&self) -> u32 {
        self.nx
    }

    #[inline]
    pub fn ny(&self) -> u32 {
        self.ny
    }

    #[inline]
    pub fn nz(&self) -> u32 {
        self.nz
    }

    #[inline]
    pub fn dx(&self) -> f32 {
        self.dx
    }

    #[inline]
    pub fn dy(&self) -> f32 {
        self.dy
    }

    #[inline]
    pub fn dz(&self) -> f32 {
        self.dz
    }

    #[inline]
    pub fn size(&self) -> [u32; 3] {
        [self.nx, self.ny, self.nz]
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.nx as usize * self.ny as usize * self.nz as usize
    }
}
