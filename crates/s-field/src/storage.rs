//! Shared storage implementation for field containers.

use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct FieldStorage<T> {
    data: Vec<T>,
}

impl<T> FieldStorage<T> {
    /// Creates a storage from an existing vector.
    #[inline]
    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
    }

    /// Returns the number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns immutable slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns mutable slice.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Consumes the storage and returns the inner Vec.
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Immutable iterator.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    /// Mutable iterator.
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }

    /// Returns a reference without bounds checking.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `index < len()`.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.data.get_unchecked(index)
    }

    /// Returns a mutable reference without bounds checking.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `index < len()`.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.data.get_unchecked_mut(index)
    }

    /// Swaps two elements.
    #[inline]
    pub fn swap(&mut self, a: usize, b: usize) {
        self.data.swap(a, b);
    }

    /// Clears the storage.
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns the allocated capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Returns memory usage in bytes.
    #[inline]
    pub fn memory_size(&self) -> usize {
        self.data.len() * std::mem::size_of::<T>()
    }

    /// Maps all elements into another storage.
    #[inline]
    pub fn map<U, F>(&self, mut f: F) -> FieldStorage<U>
    where
        F: FnMut(&T) -> U,
    {
        FieldStorage {
            data: self.data.iter().map(&mut f).collect(),
        }
    }

    /// Applies a function to each element in place.
    #[inline]
    pub fn map_in_place<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        for value in &mut self.data {
            f(value);
        }
    }
}

impl<T: Clone> FieldStorage<T> {
    /// Creates a storage of `size` initialized with `value`.
    #[inline]
    pub fn new(size: usize, value: T) -> Self {
        Self {
            data: vec![value; size],
        }
    }

    /// Fills all elements with `value`.
    #[inline]
    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }
}

impl<T> Index<usize> for FieldStorage<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for FieldStorage<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
