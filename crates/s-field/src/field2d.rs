//! Two-dimensional field container.

use crate::{Field, FieldStorage, Grid2D};

#[derive(Clone, Debug)]
pub struct Field2D<T> {
    grid: Grid2D,
    storage: FieldStorage<T>,
}

impl<T: Clone> Field2D<T> {
    /// Creates a new field filled with `value`.
    #[inline]
    pub fn new(grid: Grid2D, value: T) -> Self {
        Self {
            storage: FieldStorage::new(grid.len(), value),
            grid,
        }
    }

    /// Creates a field from an existing Vec.
    pub fn from_vec(grid: Grid2D, data: Vec<T>) -> Self {
        assert_eq!(grid.len(), data.len(), "Field2D size mismatch");

        Self {
            grid,
            storage: FieldStorage::from_vec(data),
        }
    }

    /// Fills every cell with the same value.
    #[inline]
    pub fn fill(&mut self, value: T) {
        self.storage.fill(value);
    }
}

impl<T> Field2D<T> {
    #[inline]
    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.grid.width() {
            return None;
        }

        if y >= self.grid.height() {
            return None;
        }

        Some(y * self.grid.width() + x)
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.index(x, y).map(|i| &self.storage[i])
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.index(x, y).map(move |i| &mut self.storage[i])
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: T) -> bool {
        if let Some(i) = self.index(x, y) {
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
    pub fn shape(&self) -> [usize; 2] {
        self.grid.size()
    }

    #[inline]
    pub fn spacing(&self) -> [f32; 2] {
        self.grid.spacing()
    }
    #[inline]

    pub fn grid(&self) -> &Grid2D {
        &self.grid
    }
    #[inline]
    pub fn swap(&mut self, a: (usize, usize), b: (usize, usize)) -> bool {
        let ia = match self.index(a.0, a.1) {
            Some(v) => v,
            None => return false,
        };

        let ib = match self.index(b.0, b.1) {
            Some(v) => v,
            None => return false,
        };

        self.storage.swap(ia, ib);

        true
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
    pub fn map<U, F>(&self, f: F) -> Field2D<U>
    where
        F: FnMut(&T) -> U,
    {
        Field2D {
            grid: self.grid,
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
}

impl<T> Field for Field2D<T> {
    type Item = T;

    type Grid = Grid2D;

    #[inline]
    fn grid(&self) -> &Grid2D {
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

impl<T: Default> Field2D<T> {
    #[inline]

    pub fn clear(&mut self) {
        self.storage.clear();
    }
}
