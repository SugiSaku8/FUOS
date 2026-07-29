//! Ocean dataset.

use std::collections::BTreeMap;

use crate::{Coordinates, Field4D, Metadata, SalinityField, TemperatureField, VelocityField};

/// Complete ocean dataset.
///
/// This is the central data container used throughout SOS.
///
/// It stores:
///
/// - Coordinates
/// - Metadata
/// - Velocity
/// - Temperature
/// - Salinity
///
/// and optional additional scalar fields.
#[derive(Clone, Debug)]
pub struct OceanField {
    coordinates: Coordinates,
    metadata: Metadata,

    velocity: VelocityField,
    temperature: TemperatureField,
    salinity: SalinityField,

    scalar_fields: BTreeMap<String, Field4D<f32>>,
}

impl OceanField {
    /// Creates a new ocean dataset.
    pub fn new(
        coordinates: Coordinates,
        metadata: Metadata,
        velocity: VelocityField,
        temperature: TemperatureField,
        salinity: SalinityField,
    ) -> Self {
        Self {
            coordinates,
            metadata,
            velocity,
            temperature,
            salinity,
            scalar_fields: BTreeMap::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Coordinates
    // -------------------------------------------------------------------------

    #[inline]
    pub fn coordinates(&self) -> &Coordinates {
        &self.coordinates
    }

    #[inline]
    pub fn coordinates_mut(&mut self) -> &mut Coordinates {
        &mut self.coordinates
    }

    // -------------------------------------------------------------------------
    // Metadata
    // -------------------------------------------------------------------------

    #[inline]
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    #[inline]
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    // -------------------------------------------------------------------------
    // Velocity
    // -------------------------------------------------------------------------

    #[inline]
    pub fn velocity(&self) -> &VelocityField {
        &self.velocity
    }

    #[inline]
    pub fn velocity_mut(&mut self) -> &mut VelocityField {
        &mut self.velocity
    }

    // -------------------------------------------------------------------------
    // Temperature
    // -------------------------------------------------------------------------

    #[inline]
    pub fn temperature(&self) -> &TemperatureField {
        &self.temperature
    }

    #[inline]
    pub fn temperature_mut(&mut self) -> &mut TemperatureField {
        &mut self.temperature
    }

    // -------------------------------------------------------------------------
    // Salinity
    // -------------------------------------------------------------------------

    #[inline]
    pub fn salinity(&self) -> &SalinityField {
        &self.salinity
    }

    #[inline]
    pub fn salinity_mut(&mut self) -> &mut SalinityField {
        &mut self.salinity
    }

    // -------------------------------------------------------------------------
    // Extra scalar fields
    // -------------------------------------------------------------------------

    /// Inserts or replaces an additional scalar field.
    #[inline]
    pub fn insert_scalar_field<S>(&mut self, name: S, field: Field4D<f32>)
    where
        S: Into<String>,
    {
        self.scalar_fields.insert(name.into(), field);
    }

    /// Returns an additional scalar field.
    #[inline]
    pub fn scalar_field(&self, name: &str) -> Option<&Field4D<f32>> {
        self.scalar_fields.get(name)
    }

    /// Returns a mutable additional scalar field.
    #[inline]
    pub fn scalar_field_mut(&mut self, name: &str) -> Option<&mut Field4D<f32>> {
        self.scalar_fields.get_mut(name)
    }

    /// Removes an additional scalar field.
    #[inline]
    pub fn remove_scalar_field(&mut self, name: &str) -> Option<Field4D<f32>> {
        self.scalar_fields.remove(name)
    }

    /// Returns all extra scalar fields.
    #[inline]
    pub fn scalar_fields(&self) -> &BTreeMap<String, Field4D<f32>> {
        &self.scalar_fields
    }

    #[inline]
    pub fn has_scalar_field(&self, name: &str) -> bool {
        self.scalar_fields.contains_key(name)
    }
}
