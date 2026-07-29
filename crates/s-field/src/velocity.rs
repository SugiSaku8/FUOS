//! Velocity field.
use crate::{Field4D, Grid4D};
/// Three-dimensional velocity components.
///
/// Components:
///
/// - u : east-west (x)
/// - v : north-south (y)
/// - w : vertical (z)
#[derive(Clone, Debug)]
pub struct VelocityField {
    u: Field4D<f32>,
    v: Field4D<f32>,
    w: Field4D<f32>,
}

impl VelocityField {
    /// Creates velocity fields initialized to zero.
    #[inline]
    pub fn new(grid: Grid4D) -> Self {
        Self {
            u: Field4D::new(grid, 0.0),
            v: Field4D::new(grid, 0.0),
            w: Field4D::new(grid, 0.0),
        }
    }

    /// Creates a velocity field from three components.
    ///
    /// # Panics
    ///
    /// Panics if the component grids differ.
    pub fn from_fields(u: Field4D<f32>, v: Field4D<f32>, w: Field4D<f32>) -> Self {
        assert_eq!(u.grid().size(), v.grid().size());
        assert_eq!(u.grid().size(), w.grid().size());

        Self { u, v, w }
    }

    #[inline]
    pub fn grid(&self) -> &Grid4D {
        self.u.grid()
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
    pub fn u_mut(&mut self) -> &mut Field4D<f32> {
        &mut self.u
    }

    #[inline]
    pub fn v_mut(&mut self) -> &mut Field4D<f32> {
        &mut self.v
    }

    #[inline]
    pub fn w_mut(&mut self) -> &mut Field4D<f32> {
        &mut self.w
    }

    /// Sets every component to zero.
    #[inline]
    pub fn clear(&mut self) {
        self.u.fill(0.0);
        self.v.fill(0.0);
        self.w.fill(0.0);
    }

    /// Returns the velocity magnitude.
    #[inline]
    pub fn magnitude(&self, t: usize, z: usize, y: usize, x: usize) -> Option<f32> {
        let u = *self.u.get(t, z, y, x)?;
        let v = *self.v.get(t, z, y, x)?;
        let w = *self.w.get(t, z, y, x)?;

        Some((u * u + v * v + w * w).sqrt())
    }

    /// Returns one velocity vector.
    #[inline]
    pub fn vector(&self, t: usize, z: usize, y: usize, x: usize) -> Option<[f32; 3]> {
        Some([
            *self.u.get(t, z, y, x)?,
            *self.v.get(t, z, y, x)?,
            *self.w.get(t, z, y, x)?,
        ])
    }

    #[inline]
    pub fn shape(&self) -> [usize; 4] {
        self.grid().size()
    }

    #[inline]
    pub fn spacing(&self) -> [f32; 4] {
        self.grid().spacing()
    }
}
