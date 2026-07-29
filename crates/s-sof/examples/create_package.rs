use std::path::PathBuf;

use s_field::{
    Coordinates, Grid4D, Metadata, OceanField, SalinityField, TemperatureField, VelocityField,
};

use s_sof::builder::SofPackageBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating SOF package...");

    //
    // Grid
    //
    let grid = Grid4D::new([5, 2, 2, 1], [1.0, 1.0, 10.0, 1.0]);

    //
    // Coordinates
    //
    let coordinates = Coordinates::new(
        grid.clone(),
        vec![35.0, 35.1],
        vec![138.5, 138.6, 138.7, 138.8, 138.9],
        vec![0.0, 10.0],
        vec![0.0],
    );

    //
    // OceanField
    //
    let metadata = Metadata::new();

    let velocity = VelocityField::new(grid.clone());

    let temperature = TemperatureField::new(grid.clone());

    let salinity = SalinityField::new(grid.clone());

    let field = OceanField::new(coordinates, metadata, velocity, temperature, salinity);

    //
    // Package
    //
    let output = PathBuf::from("dummy_ocean.sof");

    SofPackageBuilder::new("Dummy Suruga Bay Ocean")
        .boundary(field)
        .build(output)?;

    println!("SOF package created.");

    Ok(())
}
