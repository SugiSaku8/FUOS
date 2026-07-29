//! Grid definitions.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid2D {
    size: [usize; 2],
    spacing: [f32; 2],
}

impl Grid2D {
    #[inline]
    pub const fn new(size: [usize; 2], spacing: [f32; 2]) -> Self {
        Self { size, spacing }
    }

    #[inline]
    pub const fn width(&self) -> usize {
        self.size[0]
    }

    #[inline]
    pub const fn height(&self) -> usize {
        self.size[1]
    }

    #[inline]
    pub const fn dx(&self) -> f32 {
        self.spacing[0]
    }

    #[inline]
    pub const fn dy(&self) -> f32 {
        self.spacing[1]
    }

    #[inline]
    pub const fn size(&self) -> [usize; 2] {
        self.size
    }

    #[inline]
    pub const fn spacing(&self) -> [f32; 2] {
        self.spacing
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.width() * self.height()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid3D {
    size: [usize; 3],
    spacing: [f32; 3],
}

impl Grid3D {
    #[inline]
    pub const fn new(size: [usize; 3], spacing: [f32; 3]) -> Self {
        Self { size, spacing }
    }

    #[inline]
    pub const fn width(&self) -> usize {
        self.size[0]
    }

    #[inline]
    pub const fn height(&self) -> usize {
        self.size[1]
    }

    #[inline]
    pub const fn depth(&self) -> usize {
        self.size[2]
    }

    #[inline]
    pub const fn dx(&self) -> f32 {
        self.spacing[0]
    }

    #[inline]
    pub const fn dy(&self) -> f32 {
        self.spacing[1]
    }

    #[inline]
    pub const fn dz(&self) -> f32 {
        self.spacing[2]
    }

    #[inline]
    pub const fn size(&self) -> [usize; 3] {
        self.size
    }

    #[inline]
    pub const fn spacing(&self) -> [f32; 3] {
        self.spacing
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.width() * self.height() * self.depth()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid4D {
    size: [usize; 4],
    spacing: [f32; 4],
}

impl Grid4D {
    #[inline]
    pub const fn new(size: [usize; 4], spacing: [f32; 4]) -> Self {
        Self { size, spacing }
    }

    #[inline]
    pub const fn width(&self) -> usize {
        self.size[0]
    }

    #[inline]
    pub const fn height(&self) -> usize {
        self.size[1]
    }

    #[inline]
    pub const fn depth(&self) -> usize {
        self.size[2]
    }

    #[inline]
    pub const fn time(&self) -> usize {
        self.size[3]
    }

    #[inline]
    pub const fn dx(&self) -> f32 {
        self.spacing[0]
    }

    #[inline]
    pub const fn dy(&self) -> f32 {
        self.spacing[1]
    }

    #[inline]
    pub const fn dz(&self) -> f32 {
        self.spacing[2]
    }

    #[inline]
    pub const fn dt(&self) -> f32 {
        self.spacing[3]
    }

    #[inline]
    pub const fn size(&self) -> [usize; 4] {
        self.size
    }

    #[inline]
    pub const fn spacing(&self) -> [f32; 4] {
        self.spacing
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.width() * self.height() * self.depth() * self.time()
    }
}
