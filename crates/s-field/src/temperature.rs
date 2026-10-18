//! Temperature field.

use crate::{Field, Field4D, Grid4D};

/// Water temperature field.
///
/// Unit:
/// - degree Celsius (°C)
#[derive(Clone, Debug)]
pub struct TemperatureField {
    field: Field4D<f32>,
}

impl TemperatureField {
    /// Creates a temperature field initialized to 0°C.
    #[inline]
    pub fn new(grid: Grid4D) -> Self {
        Self {
            field: Field4D::new(grid, 0.0),
        }
    }

    /// Wraps an existing field.
    #[inline]
    pub fn from_field(field: Field4D<f32>) -> Self {
        Self { field }
    }

    #[inline]
    pub fn field(&self) -> &Field4D<f32> {
        &self.field
    }

    #[inline]
    pub fn field_mut(&mut self) -> &mut Field4D<f32> {
        &mut self.field
    }

    #[inline]
    pub fn grid(&self) -> &Grid4D {
        self.field.grid()
    }

    #[inline]
    pub fn shape(&self) -> [usize; 4] {
        self.grid().size()
    }

    #[inline]
    pub fn spacing(&self) -> [f32; 4] {
        self.grid().spacing()
    }

    #[inline]
    pub fn get(&self, t: usize, z: usize, y: usize, x: usize) -> Option<f32> {
        self.field.get(t, z, y, x).copied()
    }

    #[inline]
    pub fn set(&mut self, t: usize, z: usize, y: usize, x: usize, value: f32) -> bool {
        self.field.set(t, z, y, x, value)
    }

    #[inline]
    pub fn fill(&mut self, value: f32) {
        self.field.fill(value);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.field.fill(0.0);
    }

    /// Returns the minimum temperature.
    pub fn min(&self) -> Option<f32> {
        self.field.as_slice().iter().copied().reduce(f32::min)
    }

    /// Returns the maximum temperature.
    pub fn max(&self) -> Option<f32> {
        self.field.as_slice().iter().copied().reduce(f32::max)
    }

    /// Returns the average temperature.
    pub fn average(&self) -> Option<f32> {
        let slice = self.field.as_slice();

        if slice.is_empty() {
            return None;
        }

        let sum: f32 = slice.iter().sum();

        Some(sum / slice.len() as f32)
    }
}

