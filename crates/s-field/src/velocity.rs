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
    /// Samples velocity using 4D multilinear interpolation.
    ///
    /// Interpolates across longitude, latitude, depth, and time.
    ///
    /// Returns None when:
    /// - Spatial coordinates are outside the dataset.
    /// - The requested time is outside the available time range.
    /// - Coordinates are invalid or non-monotonic.
    /// - A required velocity value is non-finite.
    pub fn sample_interpolated(
        &self,
        longitude: f64,
        latitude: f64,
        depth: f32,
        time: f64,
    ) -> Option<[f32; 3]> {
        let coord = self.coordinates.as_ref()?;

        if !longitude.is_finite()
            || !latitude.is_finite()
            || !depth.is_finite()
            || !time.is_finite()
        {
            return None;
        }

        if !coord.contains(longitude, latitude, depth) {
            return None;
        }

        // Find the two surrounding grid indices on each axis.
        let (x0, x1, wx) = bracket_axis(coord.longitude(), longitude)?;

        let (y0, y1, wy) = bracket_axis(coord.latitude(), latitude)?;

        let depth_values: Vec<f64> = coord.depth().iter().map(|&value| value as f64).collect();

        let (z0, z1, wz) = bracket_axis(&depth_values, depth as f64)?;

        // A single time slice represents a time-independent field.
        let (t0, t1, wt) = if coord.time_steps() == 1 {
            (0, 0, 0.0)
        } else {
            bracket_axis(coord.time(), time)?
        };

        let xs = [(x0, 1.0 - wx), (x1, wx)];
        let ys = [(y0, 1.0 - wy), (y1, wy)];
        let zs = [(z0, 1.0 - wz), (z1, wz)];
        let ts = [(t0, 1.0 - wt), (t1, wt)];

        let mut result = [0.0_f64; 3];

        for &(t, tw) in &ts {
            for &(z, zw) in &zs {
                for &(y, yw) in &ys {
                    for &(x, xw) in &xs {
                        let weight = tw * zw * yw * xw;

                        // Avoid reading unnecessary corners when a query
                        // lies exactly on a grid coordinate.
                        if weight == 0.0 {
                            continue;
                        }

                        let velocity = self.velocity(t, x, y, z)?;

                        for component in 0..3 {
                            let value = velocity[component];

                            if !value.is_finite() {
                                return None;
                            }

                            result[component] += value as f64 * weight;
                        }
                    }
                }
            }
        }

        let interpolated = [result[0] as f32, result[1] as f32, result[2] as f32];

        if interpolated.iter().all(|value| value.is_finite()) {
            Some(interpolated)
        } else {
            None
        }
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

/// Finds the surrounding indices and interpolation weight.
///
/// Supports ascending and descending monotonic axes.
///
/// Returns:
/// (lower_index, upper_index, interpolation_weight)
///
/// The weight is 0 at the first index and 1 at the second index.
fn bracket_axis(axis: &[f64], value: f64) -> Option<(usize, usize, f64)> {
    if axis.is_empty() || !value.is_finite() {
        return None;
    }

    if axis.iter().any(|coordinate| !coordinate.is_finite()) {
        return None;
    }

    // A single-coordinate axis has no interval.
    if axis.len() == 1 {
        if value == axis[0] {
            return Some((0, 0, 0.0));
        }

        return None;
    }

    let ascending = axis[1] > axis[0];

    // Reject duplicate or non-monotonic coordinates.
    for pair in axis.windows(2) {
        if ascending {
            if pair[1] <= pair[0] {
                return None;
            }
        } else if pair[1] >= pair[0] {
            return None;
        }
    }

    // Reject values outside the axis range.
    let first = axis[0];
    let last = axis[axis.len() - 1];

    if ascending {
        if value < first || value > last {
            return None;
        }
    } else if value > first || value < last {
        return None;
    }

    // Exact endpoints need no interpolation.
    if value == first {
        return Some((0, 0, 0.0));
    }

    if value == last {
        let index = axis.len() - 1;
        return Some((index, index, 0.0));
    }

    // Locate the interval containing the requested value.
    for i in 0..axis.len() - 1 {
        let a = axis[i];
        let b = axis[i + 1];

        let inside = if ascending {
            value >= a && value <= b
        } else {
            value <= a && value >= b
        };

        if inside {
            let weight = (value - a) / (b - a);

            return Some((i, i + 1, weight));
        }
    }

    None
}
