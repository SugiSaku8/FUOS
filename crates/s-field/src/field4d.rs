//! Four-dimensional field container.
//
// Layout:
//
// (time, depth, y, x)
//
// index = (((t * depth + z) * height + y) * width + x)

use crate::{Field, FieldStorage, Grid4D};

#[derive(Clone, Debug)]
pub struct Field4D<T> {
    grid: Grid4D,
    storage: FieldStorage<T>,
}

impl<T: Clone> Field4D<T> {
    #[inline]
    pub fn new(grid: Grid4D, value: T) -> Self {
        Self {
            storage: FieldStorage::new(grid.len(), value),
            grid,
        }
    }

    pub fn from_vec(grid: Grid4D, data: Vec<T>) -> Self {
        assert_eq!(grid.len(), data.len(), "Field4D size mismatch",);

        Self {
            grid,
            storage: FieldStorage::from_vec(data),
        }
    }

    #[inline]
    pub fn fill(&mut self, value: T) {
        self.storage.fill(value);
    }
}

impl<T> Field4D<T> {
    #[inline]
    fn index(&self, t: usize, z: usize, y: usize, x: usize) -> Option<usize> {
        if x >= self.grid.width() {
            return None;
        }

        if y >= self.grid.height() {
            return None;
        }

        if z >= self.grid.depth() {
            return None;
        }

        if t >= self.grid.time() {
            return None;
        }

        Some((((t * self.grid.depth() + z) * self.grid.height() + y) * self.grid.width()) + x)
    }

    #[inline]
    pub fn get(&self, t: usize, z: usize, y: usize, x: usize) -> Option<&T> {
        self.index(t, z, y, x).map(|i| &self.storage[i])
    }

    #[inline]
    pub fn get_mut(&mut self, t: usize, z: usize, y: usize, x: usize) -> Option<&mut T> {
        self.index(t, z, y, x).map(move |i| &mut self.storage[i])
    }

    #[inline]
    pub fn set(&mut self, t: usize, z: usize, y: usize, x: usize, value: T) -> bool {
        if let Some(i) = self.index(t, z, y, x) {
            self.storage[i] = value;
            true
        } else {
            false
        }
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
    pub fn depth(&self) -> usize {
        self.grid.depth()
    }

    #[inline]
    pub fn time_steps(&self) -> usize {
        self.grid.time()
    }

    #[inline]
    pub fn shape(&self) -> [usize; 4] {
        self.grid.size()
    }

    #[inline]
    pub fn spacing(&self) -> [f32; 4] {
        self.grid.spacing()
    }

    #[inline]

    pub fn grid(&self) -> &Grid4D {
        &self.grid
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.storage.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.storage.iter_mut()
    }

    #[inline]
    pub fn map<U, F>(&self, f: F) -> Field4D<U>
    where
        F: FnMut(&T) -> U,
    {
        Field4D {
            grid: *self.grid(),
            storage: self.storage.map(f),
        }
    }

    #[inline]
    pub fn map_in_place<F>(&mut self, f: F)
    where
        F: FnMut(&mut T),
    {
        self.storage.map_in_place(f);
    }

    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.storage.into_vec()
    }

    /// Returns the contiguous storage for one time step.
    #[inline]
    pub fn time_slice(&self, t: usize) -> Option<&[T]> {
        if t >= self.time_steps() {
            return None;
        }

        let len = self.width() * self.height() * self.depth();

        let start = t * len;

        Some(&self.as_slice()[start..start + len])
    }
    #[inline]
    pub fn time_slice_mut(&mut self, t: usize) -> Option<&mut [T]> {
        if t >= self.time_steps() {
            return None;
        }

        let len = self.width() * self.height() * self.depth();

        let start = t * len;

        Some(&mut self.as_mut_slice()[start..start + len])
    }
}

impl<T> Field for Field4D<T> {
    type Item = T;

    type Grid = Grid4D;

    #[inline]
    fn grid(&self) -> &Grid4D {
        &self.grid
    }

    #[inline]
    fn len(&self) -> usize {
        self.storage.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    #[inline]
    fn as_slice(&self) -> &[T] {
        self.storage.as_slice()
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.storage.as_mut_slice()
    }
}

impl<T: Default> Field4D<T> {
    #[inline]

    pub fn clear(&mut self) {
        self.storage.clear();
    }
}

impl<T> Field4D<T> {
    /// Returns the contiguous data slice without requiring the `Field` trait
    /// to be imported by downstream crates.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.storage.as_slice()
    }

    /// Returns the mutable contiguous data slice without requiring the `Field`
    /// trait to be imported by downstream crates.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.storage.as_mut_slice()
    }

    /// Backwards-compatible alias for the contiguous data slice.
    #[inline]
    pub fn data(&self) -> &[T] {
        self.as_slice()
    }

    /// Backwards-compatible alias for the mutable contiguous data slice.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

#[cfg(test)]
mod slice_api_tests {
    use super::*;
    use crate::Grid4D;

    #[test]
    fn inherent_slice_accessors_work_without_trait_dispatch() {
        let grid = Grid4D::new([1, 1, 2, 2], [1.0; 4]);
        let mut field = Field4D::from_vec(grid, vec![1_u32, 2, 3, 4]);
        assert_eq!(field.as_slice(), &[1, 2, 3, 4]);
        field.as_mut_slice()[2] = 9;
        assert_eq!(field.data(), &[1, 2, 9, 4]);
    }
}
