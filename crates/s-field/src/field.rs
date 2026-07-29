//! Common field trait.

/// Common interface implemented by every field container.
pub trait Field {
    /// Stored value type.
    type Item;

    /// Grid type.
    type Grid;

    /// Returns the grid.
    fn grid(&self) -> &Self::Grid;

    /// Number of elements.
    fn len(&self) -> usize;

    /// Returns true if empty.
    fn is_empty(&self) -> bool;

    /// Returns contiguous immutable storage.
    fn as_slice(&self) -> &[Self::Item];

    /// Returns contiguous mutable storage.
    fn as_mut_slice(&mut self) -> &mut [Self::Item];

    /// Returns the allocated memory size in bytes.
    #[inline]
    fn memory_size(&self) -> usize {
        self.len() * std::mem::size_of::<Self::Item>()
    }
}
