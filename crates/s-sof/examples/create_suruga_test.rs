use std::path::Path;

use s_field::{
    Coordinates, Field, Grid4D, Metadata, OceanField, SalinityField, TemperatureField,
    VelocityField,
};

use s_sof::builder::SofPackageBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //
    // Grid
    //

    let grid = Grid4D::new([32, 32, 10, 1], [0.01, 0.01, 10.0, 1.0]);

    //
    // Coordinates
    //

    let latitude: Vec<f64> = (0..32).map(|i| 34.5 + i as f64 * 0.01).collect();

    let longitude: Vec<f64> = (0..32).map(|i| 138.5 + i as f64 * 0.01).collect();

    let depth: Vec<f32> = (0..10).map(|i| i as f32 * 50.0).collect();

    let time = vec![0.0];

    let coordinates = Coordinates::new(grid.clone(), latitude, longitude, depth, time);

    //
    // Velocity
    //

    let mut velocity = VelocityField::with_coordinates(grid.clone(), coordinates.clone());

    // Kuroshio-like synthetic flow

    for value in velocity.u_mut().as_mut_slice() {
        *value = 1.0;
    }

    for value in velocity.v_mut().as_mut_slice() {
        *value = 0.2;
    }

    for value in velocity.w_mut().as_mut_slice() {
        *value = 0.0;
    }

    //
    // Temperature
    //

    let mut temperature = TemperatureField::new(grid.clone());

    for value in temperature.field_mut().as_mut_slice() {
        *value = 20.0;
    }

    //
    // Salinity
    //

    let mut salinity = SalinityField::new(grid.clone());

    for value in salinity.field_mut().as_mut_slice() {
        *value = 34.5;
    }

    //
    // Metadata
    //

    let mut metadata = Metadata::new();

    metadata.set_dataset_name("Synthetic Suruga Bay Dataset");

    metadata.set_title("Suruga Bay Test Ocean Field");

    metadata.set_institution("Suruga Ocean Simulator");

    metadata.set_source("Synthetic generator");

    //
    // OceanField
    //

    let ocean = OceanField::new(
        coordinates.clone(),
        metadata,
        velocity,
        temperature,
        salinity,
    );
    //
    // Create SOF package
    //

    let output = Path::new("./data/suruga_test.sof");

    let _package = SofPackageBuilder::new("Suruga Bay Test")
        .boundary(ocean.clone())
        .initial(ocean.clone())
        .restart(ocean)
        .grid(
            grid.clone(),
            coordinates.latitude().iter().map(|v| *v as f32).collect(),
            coordinates.longitude().iter().map(|v| *v as f32).collect(),
            coordinates.depth().to_vec(),
            vec![0.0; 32 * 32],
        )
        .forcing(vec![0.0; 32 * 32], vec![0.0; 32 * 32], vec![0.0; 32 * 32])
        .build(output)?;

    Ok(())
}
