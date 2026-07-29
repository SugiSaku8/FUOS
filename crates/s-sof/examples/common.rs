use s_field::{
    Coordinates, Field, Grid4D, Metadata, OceanField, SalinityField, TemperatureField,
    VelocityField,
};

pub fn create_dummy_field() -> OceanField {
    //
    // Grid
    //

    let grid = Grid4D::new([5, 2, 2, 1], [1.0, 1.0, 10.0, 1.0]);

    //
    // Coordinates
    //

    let latitude = vec![35.0, 35.1];

    let longitude = vec![138.5, 138.6, 138.7, 138.8, 138.9];

    let depth = vec![0.0, 50.0];

    let time = vec![0.0];

    let coordinates = Coordinates::new(grid, latitude, longitude, depth, time);

    //
    // Fields
    //

    let mut velocity = VelocityField::new(grid);

    let mut temperature = TemperatureField::new(grid);

    let mut salinity = SalinityField::new(grid);

    //
    // Fill dummy values
    //

    for v in velocity.u_mut().as_mut_slice() {
        *v = 0.1;
    }

    for v in velocity.v_mut().as_mut_slice() {
        *v = 0.2;
    }

    for v in velocity.w_mut().as_mut_slice() {
        *v = 0.0;
    }

    for v in temperature.field_mut().as_mut_slice() {
        *v = 20.0;
    }

    for v in salinity.field_mut().as_mut_slice() {
        *v = 34.5;
    }

    //
    // Metadata
    //

    let mut metadata = Metadata::new();

    metadata.set_dataset_name("dummy_ocean");

    metadata.set_title("SOS Dummy Ocean Dataset");

    metadata.set_source("create_package example");

    //
    // OceanField
    //

    OceanField::new(coordinates, metadata, velocity, temperature, salinity)
}
