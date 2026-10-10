//! Boundary condition fields.

use crate::VelocityField;

/// Ocean boundary velocity.
///
/// Represents velocity imposed at domain boundaries.
#[derive(Clone, Debug)]
pub struct BoundaryField {
    /// Western boundary.
    ///
    /// Used for Kuroshio inflow.
    pub west: VelocityField,

    /// Eastern boundary.
    pub east: VelocityField,

    /// Northern boundary.
    pub north: VelocityField,

    /// Southern boundary.
    pub south: VelocityField,
}

impl BoundaryField {
    /// Create empty boundary field.
    pub fn new(
        west: VelocityField,
        east: VelocityField,
        north: VelocityField,
        south: VelocityField,
    ) -> Self {
        Self {
            west,
            east,
            north,
            south,
        }
    }

    /// Create zero boundary condition.
    pub fn zeros(grid: crate::Grid4D) -> Self {
        Self {
            west: VelocityField::new(grid.clone()),

            east: VelocityField::new(grid.clone()),

            north: VelocityField::new(grid.clone()),

            south: VelocityField::new(grid),
        }
    }
}
