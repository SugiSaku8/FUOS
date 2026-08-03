//! Coordinate axes for an ocean dataset.

use crate::Grid4D;

/// Coordinate axes for a 4D ocean dataset.
///
/// Axis order:
///
/// time
/// depth
/// latitude
/// longitude
///
#[derive(Clone, Debug)]
pub struct Coordinates {
    grid: Grid4D,

    latitude: Vec<f64>,
    longitude: Vec<f64>,
    depth: Vec<f32>,
    time: Vec<f64>,
}

impl Coordinates {
    /// Creates a new coordinate system.
    ///
    /// # Panics
    ///
    /// Panics if axis lengths do not match the supplied grid.
    pub fn new(
        grid: Grid4D,
        latitude: Vec<f64>,
        longitude: Vec<f64>,
        depth: Vec<f32>,
        time: Vec<f64>,
    ) -> Self {
        assert_eq!(
            grid.width(),
            longitude.len(),
            "longitude axis size mismatch"
        );

        assert_eq!(grid.height(), latitude.len(), "latitude axis size mismatch");

        assert_eq!(grid.depth(), depth.len(), "depth axis size mismatch");

        assert_eq!(grid.time(), time.len(), "time axis size mismatch");

        Self {
            grid,
            latitude,
            longitude,
            depth,
            time,
        }
    }

    #[inline]
    pub fn grid(&self) -> &Grid4D {
        &self.grid
    }

    #[inline]
    pub fn latitude(&self) -> &[f64] {
        &self.latitude
    }

    #[inline]
    pub fn longitude(&self) -> &[f64] {
        &self.longitude
    }

    #[inline]
    pub fn depth(&self) -> &[f32] {
        &self.depth
    }

    #[inline]
    pub fn time(&self) -> &[f64] {
        &self.time
    }

    #[inline]
    pub fn latitude_at(&self, index: usize) -> Option<f64> {
        self.latitude.get(index).copied()
    }

    #[inline]
    pub fn longitude_at(&self, index: usize) -> Option<f64> {
        self.longitude.get(index).copied()
    }

    #[inline]
    pub fn depth_at(&self, index: usize) -> Option<f32> {
        self.depth.get(index).copied()
    }

    #[inline]
    pub fn time_at(&self, index: usize) -> Option<f64> {
        self.time.get(index).copied()
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.grid.width()
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.grid.height()
    }

    #[inline]
    pub fn depth_levels(&self) -> usize {
        self.grid.depth()
    }

    #[inline]
    pub fn time_steps(&self) -> usize {
        self.grid.time()
    }

    /// Returns true if coordinate axes are empty.
    pub fn is_empty(&self) -> bool {
        self.latitude.is_empty()
            || self.longitude.is_empty()
            || self.depth.is_empty()
            || self.time.is_empty()
    }

    /// Returns true if coordinate is inside this dataset domain.
    ///
    /// Supports both ascending and descending axes.
    pub fn contains(&self, longitude: f64, latitude: f64, depth: f32) -> bool {
        if self.is_empty() {
            return false;
        }

        let lon_min = self.longitude[0].min(self.longitude[self.longitude.len() - 1]);

        let lon_max = self.longitude[0].max(self.longitude[self.longitude.len() - 1]);

        let lat_min = self.latitude[0].min(self.latitude[self.latitude.len() - 1]);

        let lat_max = self.latitude[0].max(self.latitude[self.latitude.len() - 1]);

        let dep_min = self.depth[0].min(self.depth[self.depth.len() - 1]);

        let dep_max = self.depth[0].max(self.depth[self.depth.len() - 1]);

        longitude >= lon_min
            && longitude <= lon_max
            && latitude >= lat_min
            && latitude <= lat_max
            && depth >= dep_min
            && depth <= dep_max
    }

    /// Returns nearest indices.
    ///
    /// Order:
    ///
    /// longitude
    /// latitude
    /// depth
    /// time
    ///
    pub fn nearest(
        &self,
        longitude: f64,
        latitude: f64,
        depth: f32,
        time: f64,
    ) -> (usize, usize, usize, usize) {
        (
            self.nearest_longitude(longitude),
            self.nearest_latitude(latitude),
            self.nearest_depth(depth),
            self.nearest_time(time),
        )
    }

    /// Returns nearest longitude index.
    pub fn nearest_longitude(&self, longitude: f64) -> usize {
        nearest_f64(&self.longitude, longitude)
    }

    /// Returns nearest latitude index.
    pub fn nearest_latitude(&self, latitude: f64) -> usize {
        nearest_f64(&self.latitude, latitude)
    }

    /// Returns nearest depth index.
    pub fn nearest_depth(&self, depth: f32) -> usize {
        nearest_f32(&self.depth, depth)
    }

    /// Returns nearest time index.
    pub fn nearest_time(&self, time: f64) -> usize {
        nearest_f64(&self.time, time)
    }

    /// Converts 4D indices into linear storage index.
    ///
    /// Layout:
    ///
    /// time
    ///  └ depth
    ///      └ latitude
    ///          └ longitude
    ///
    #[inline]
    pub fn grid_index(
        &self,
        time: usize,
        depth: usize,
        latitude: usize,
        longitude: usize,
    ) -> usize {
        (((time * self.depth_levels() + depth) * self.height() + latitude) * self.width())
            + longitude
    }
}

fn nearest_f64(values: &[f64], value: f64) -> usize {
    values
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (value - **a)
                .abs()
                .partial_cmp(&(value - **b).abs())
                .unwrap()
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn nearest_f32(values: &[f32], value: f32) -> usize {
    values
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (value - **a)
                .abs()
                .partial_cmp(&(value - **b).abs())
                .unwrap()
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}
