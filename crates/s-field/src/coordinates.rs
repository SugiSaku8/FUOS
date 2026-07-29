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
    /// Panics if the axis lengths do not match the supplied grid.
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
}
