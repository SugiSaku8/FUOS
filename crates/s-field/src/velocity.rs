//! Velocity field.

use crate::{Coordinates, Grid4D, ScalarField};

/// 3次元流速場.
///
/// Components:
///
/// u : east-west velocity
/// v : north-south velocity
/// w : vertical velocity
///
#[derive(Clone, Debug)]
pub struct VelocityField {
    grid: Grid4D,

    coordinates: Option<Coordinates>,

    u: ScalarField<f32>,

    v: ScalarField<f32>,

    w: ScalarField<f32>,
}

impl VelocityField {
    /// Creates empty velocity field.
    pub fn new(grid: Grid4D) -> Self {
        Self {
            u: ScalarField::zeros(grid.clone()),

            v: ScalarField::zeros(grid.clone()),

            w: ScalarField::zeros(grid.clone()),

            coordinates: None,

            grid,
        }
    }

    /// Creates velocity field with coordinates.
    pub fn with_coordinates(grid: Grid4D, coordinates: Coordinates) -> Self {
        Self {
            u: ScalarField::zeros(grid.clone()),

            v: ScalarField::zeros(grid.clone()),

            w: ScalarField::zeros(grid.clone()),

            coordinates: Some(coordinates),

            grid,
        }
    }

    #[inline]
    pub fn grid(&self) -> &Grid4D {
        &self.grid
    }

    #[inline]
    pub fn shape(&self) -> [usize; 3] {
        [self.grid.depth(), self.grid.height(), self.grid.width()]
    }

    /// Returns grid spacing.

    #[inline]

    pub fn spacing(&self) -> [f32; 4] {
        self.grid.spacing()
    }

    #[inline]
    pub fn coordinates(&self) -> Option<&Coordinates> {
        self.coordinates.as_ref()
    }

    #[inline]
    pub fn coordinates_mut(&mut self) -> Option<&mut Coordinates> {
        self.coordinates.as_mut()
    }

    #[inline]
    pub fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.coordinates = Some(coordinates);
    }

    #[inline]
    pub fn u(&self) -> &ScalarField<f32> {
        &self.u
    }

    #[inline]
    pub fn v(&self) -> &ScalarField<f32> {
        &self.v
    }

    #[inline]
    pub fn w(&self) -> &ScalarField<f32> {
        &self.w
    }

    #[inline]
    pub fn u_mut(&mut self) -> &mut ScalarField<f32> {
        &mut self.u
    }

    #[inline]
    pub fn v_mut(&mut self) -> &mut ScalarField<f32> {
        &mut self.v
    }

    #[inline]
    pub fn w_mut(&mut self) -> &mut ScalarField<f32> {
        &mut self.w
    }

    /// Returns velocity by linear index.
    pub fn velocity_at_index(&self, time: usize, index: usize) -> Option<[f32; 3]> {
        Some([
            *self.u.time_slice(time)?.get(index)?,
            *self.v.time_slice(time)?.get(index)?,
            *self.w.time_slice(time)?.get(index)?,
        ])
    }

    /// Returns velocity by grid index.
    pub fn velocity(&self, time: usize, x: usize, y: usize, z: usize) -> Option<[f32; 3]> {
        if x >= self.grid.width() || y >= self.grid.height() || z >= self.grid.depth() {
            return None;
        }

        let index = z * self.grid.width() * self.grid.height() + y * self.grid.width() + x;

        self.velocity_at_index(time, index)
    }

    /// Samples velocity from physical coordinates.
    ///
    /// Coordinates:
    ///
    /// longitude
    /// latitude
    /// depth
    ///
    pub fn sample(&self, longitude: f64, latitude: f64, depth: f32, time: f64) -> Option<[f32; 3]> {
        let coord = self.coordinates.as_ref()?;

        let x = coord.nearest_longitude(longitude);

        let y = coord.nearest_latitude(latitude);

        let z = coord.nearest_depth(depth);

        let t = coord.nearest_time(time);

        self.velocity(t, x, y, z)
    }

    /// Clears all velocity values.
    pub fn clear(&mut self) {
        self.u.fill(0.0);

        self.v.fill(0.0);

        self.w.fill(0.0);
    }
    /// Creates velocity field from existing scalar fields.
    pub fn from_fields(
        grid: Grid4D,
        u: ScalarField<f32>,
        v: ScalarField<f32>,
        w: ScalarField<f32>,
    ) -> Self {
        Self {
            grid,
            coordinates: None,
            u,
            v,
            w,
        }
    }
}
