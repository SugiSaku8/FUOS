//! Common traits for field containers.

/// Common interface implemented by all field types.
///
/// This trait intentionally exposes only functionality that is
/// dimension-independent.
///
/// Dimension-specific indexing (2D / 3D / 4D) is implemented on each field
/// individually.
pub trait Field {
    /// Element type.
    type Item;

    /// Grid type.
    type Grid;

    /// Returns the grid.
    fn grid(&self) -> &Self::Grid;

    /// Number of stored elements.
    fn len(&self) -> usize;

    /// Returns true if empty.
    fn is_empty(&self) -> bool;

    /// Immutable slice.
    fn as_slice(&self) -> &[Self::Item];

    /// Mutable slice.
    fn as_mut_slice(&mut self) -> &mut [Self::Item];

    /// Memory usage in bytes.
    #[inline]
    fn memory_size(&self) -> usize {
        self.len() * std::mem::size_of::<Self::Item>()
    }
}
