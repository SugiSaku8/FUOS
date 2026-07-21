use glam::Vec3;

use crate::field4d::Field4D;

#[derive(Clone, Debug)]
pub struct VelocityField {
    u: Field4D<f32>,
    v: Field4D<f32>,
    w: Field4D<f32>,
}

impl VelocityField {
    pub fn new(u: Field4D<f32>, v: Field4D<f32>, w: Field4D<f32>) -> Self {
        assert_eq!(u.len(), v.len());
        assert_eq!(u.len(), w.len());

        Self { u, v, w }
    }

    #[inline]
    pub fn u(&self) -> &Field4D<f32> {
        &self.u
    }

    #[inline]
    pub fn v(&self) -> &Field4D<f32> {
        &self.v
    }

    #[inline]
    pub fn w(&self) -> &Field4D<f32> {
        &self.w
    }

    #[inline]
    pub fn vector(&self, t: usize, z: usize, y: usize, x: usize) -> Option<Vec3> {
        Some(Vec3::new(
            *self.u.get(t, z, y, x)?,
            *self.v.get(t, z, y, x)?,
            *self.w.get(t, z, y, x)?,
        ))
    }

    #[inline]
    pub fn speed(&self, t: usize, z: usize, y: usize, x: usize) -> Option<f32> {
        self.vector(t, z, y, x).map(|v| v.length())
    }

    #[inline]
    pub fn dimensions(&self) -> &crate::grid::Grid4D {
        self.u.grid()
    }
}
