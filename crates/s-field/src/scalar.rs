//! Scalar field storage.

use crate::{Field, Grid4D};

#[derive(Clone, Debug)]
pub struct ScalarField<T>
where
    T: Clone,
{
    grid: Grid4D,

    data: Vec<T>,
}

impl<T> ScalarField<T>
where
    T: Clone + Default,
{
    /// Creates zero/default initialized scalar field.
    pub fn zeros(grid: Grid4D) -> Self {
        let size = grid.width() * grid.height() * grid.depth() * grid.time();

        Self {
            grid,

            data: vec![T::default(); size],
        }
    }
}

impl<T> ScalarField<T>
where
    T: Clone,
{
    pub fn replace(&mut self, data: Vec<T>) {
        assert_eq!(data.len(), self.data.len(), "scalar field size mismatch");

        self.data = data;
    }

    #[inline]
    pub fn grid(&self) -> &Grid4D {
        &self.grid
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }

    pub fn time_slice(&self, time: usize) -> Option<&[T]> {
        if time >= self.grid.time() {
            return None;
        }

        let size = self.grid.width() * self.grid.height() * self.grid.depth();

        let start = time * size;

        let end = start + size;

        Some(&self.data[start..end])
    }

    pub fn time_slice_mut(&mut self, time: usize) -> Option<&mut [T]> {
        if time >= self.grid.time() {
            return None;
        }

        let size = self.grid.width() * self.grid.height() * self.grid.depth();

        let start = time * size;

        let end = start + size;

        Some(&mut self.data[start..end])
    }
}

// Field trait

impl<T> Field for ScalarField<T>
where
    T: Clone,
{
    type Item = T;

    type Grid = Grid4D;

    fn grid(&self) -> &Self::Grid {
        &self.grid
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn as_slice(&self) -> &[Self::Item] {
        &self.data
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        &mut self.data
    }
}
