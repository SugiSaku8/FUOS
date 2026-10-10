//! SOS dataset builder.

use std::path::Path;

use anyhow::Result;

use s_field::{
    BoundaryField, Coordinates, Metadata, OceanField, SalinityField, ScalarField, TemperatureField,
    VelocityField,
};

use s_sof::builder::SofPackageBuilder;

use crate::hycom::HycomDataset;

/// Builder for Suruga Ocean Simulator dataset.
pub struct SurugaBuilder;

impl SurugaBuilder {
    /// Build SOF package from HYCOM dataset.
    pub fn build<P: AsRef<Path>>(
        output: P,
        dataset: HycomDataset,
        latitude: Vec<f64>,
        longitude: Vec<f64>,
        depth: Vec<f32>,
        time: Vec<f64>,
        bathymetry: Vec<f32>,
    ) -> Result<()> {
        let grid = dataset.grid.clone();

        //
        // Coordinates
        //

        let coordinates = Coordinates::new(
            grid.clone(),
            latitude.clone(),
            longitude.clone(),
            depth.clone(),
            time.clone(),
        );

        //
        // Metadata
        //

        let mut metadata = Metadata::new();

        metadata.set_dataset_name("Suruga Bay");

        metadata.set_title("Suruga Ocean Simulator HYCOM Dataset");

        //
        // Temperature
        //

        let temperature = TemperatureField::from_field(dataset.temperature.into_field4d());

        //
        // Salinity
        //

        let salinity = SalinityField::from_field(dataset.salinity.into_field4d());

        //
        // Velocity
        //

        let velocity = VelocityField::from_fields(
            grid.clone(),
            dataset.u,
            dataset.v,
            ScalarField::zeros(grid.clone()),
        );

        //
        // OceanField
        //

        let boundary = BoundaryField::zeros(velocity.grid().clone());

        let ocean = OceanField::new(
            coordinates,
            metadata,
            velocity,
            temperature,
            salinity,
            boundary,
        );
        //
        // Surface forcing
        //
        // MVP:
        // wind_u
        // wind_v
        // surface_heat
        //

        let size = grid.len();

        let wind_u = vec![0.0f32; size];

        let wind_v = vec![0.0f32; size];

        let surface_heat = vec![0.0f32; size];

        //
        // Grid coordinates
        //

        let latitude32 = latitude.iter().map(|v| *v as f32).collect();

        let longitude32 = longitude.iter().map(|v| *v as f32).collect();

        //
        // Build SOF
        //

        SofPackageBuilder::new("suruga")
            //
            // grid.snc
            //
            .grid(grid.clone(), latitude32, longitude32, depth, bathymetry)
            //
            // boundary.snc
            //
            .boundary(ocean.clone())
            //
            // initial.snc
            //
            .initial(ocean.clone())
            //
            // restart.snc
            //
            .restart(ocean.clone())
            //
            // forcing.snc
            //
            .forcing(wind_u, wind_v, surface_heat)
            .build(output)?;

        Ok(())
    }
}
