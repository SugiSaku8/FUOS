#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid2D {
    pub width: usize,
    pub height: usize,

    pub dx: f32,
    pub dy: f32,
}

impl Grid2D {
    pub fn new(width: usize, height: usize, dx: f32, dy: f32) -> Self {
        Self {
            width,
            height,
            dx,
            dy,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.width * self.height
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid3D {
    pub width: usize,

    pub height: usize,

    pub depth: usize,

    pub dx: f32,

    pub dy: f32,

    pub dz: f32,
}

impl Grid3D {
    pub fn new(width: usize, height: usize, depth: usize, dx: f32, dy: f32, dz: f32) -> Self {
        Self {
            width,
            height,
            depth,
            dx,
            dy,
            dz,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.width * self.height * self.depth
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid4D {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub time: usize,

    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
    pub dt: f32,
}

impl Grid4D {
    pub fn new(
        width: usize,
        height: usize,
        depth: usize,
        time: usize,
        dx: f32,
        dy: f32,
        dz: f32,
        dt: f32,
    ) -> Self {
        Self {
            width,
            height,
            depth,
            time,
            dx,
            dy,
            dz,
            dt,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.width * self.height * self.depth * self.time
    }
}
