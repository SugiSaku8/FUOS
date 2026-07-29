//! Shared contiguous storage for field types.

use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct FieldStorage<T> {
    data: Vec<T>,
}

impl<T> FieldStorage<T> {
    /// Creates storage from an existing vector.
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

    /// Returns an immutable slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns a mutable slice.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
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

    /// Swaps two elements.
    #[inline]
    pub fn swap(&mut self, a: usize, b: usize) {
        self.data.swap(a, b);
    }

    /// Consumes storage and returns the vector.
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Returns the allocated memory size in bytes.
    #[inline]
    pub fn memory_size(&self) -> usize {
        self.data.len() * std::mem::size_of::<T>()
    }
}

impl<T: Clone> FieldStorage<T> {
    /// Creates storage filled with a value.
    #[inline]
    pub fn new(len: usize, value: T) -> Self {
        Self {
            data: vec![value; len],
        }
    }

    /// Fills every element.
    #[inline]
    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }
}

impl<T: Default> FieldStorage<T> {
    /// Clears the storage by resetting every element to its default value.
    #[inline]
    pub fn clear(&mut self) {
        for value in &mut self.data {
            *value = T::default();
        }
    }
}

impl<T> FieldStorage<T> {
    /// Creates a mapped storage.
    pub fn map<U, F>(&self, mut f: F) -> FieldStorage<U>
    where
        F: FnMut(&T) -> U,
    {
        FieldStorage {
            data: self.data.iter().map(&mut f).collect(),
        }
    }

    /// Maps every element in place.
    pub fn map_in_place<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        for value in &mut self.data {
            f(value);
        }
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
