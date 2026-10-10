use anyhow::Result;

use std::sync::Arc;

use s_field::{
    BoundaryField, Coordinates, Grid4D, Metadata, OceanField, SalinityField, TemperatureField,
    VelocityField,
};

use s_metal::{MetalContext, OceanSolver};

use simulation::Simulation;

use s_io::SioWriter;

fn main() -> Result<()> {
    pollster::block_on(run())
}

async fn run() -> Result<()> {
    println!("=== Suruga Ocean Simulator ===");

    //
    // Ocean field initialization
    //

    let grid = Grid4D::new([1, 32, 64, 64], [1.0, 10.0, 0.01, 0.01]);

    let coordinates = Coordinates::new(
        grid.clone(),
        vec![0.0; 64],
        vec![0.0; 64],
        vec![0.0; 32],
        vec![0.0; 1],
    );

    let ocean = OceanField::new(
        coordinates,
        Metadata::new(),
        VelocityField::new(grid.clone()),
        TemperatureField::new(grid.clone()),
        SalinityField::new(grid.clone()),
        BoundaryField::new(
            VelocityField::new(grid.clone()),
            VelocityField::new(grid.clone()),
            VelocityField::new(grid.clone()),
            VelocityField::new(grid.clone()),
        ),
    );

    //
    // Metal simulation
    //

    let context = Arc::new(MetalContext::new());

    let solver = OceanSolver::new(context.clone());

    let mut simulation = Simulation::new(&context, ocean, solver);

    //
    // Time integration
    //

    let steps = 100;
    let dt = 0.1;

    for i in 0..steps {
        simulation.step(dt);

        println!("step={} time={}", i, simulation.time);
    }

    //
    // Download velocity from GPU
    //

    println!("Downloading velocity field...");

    let velocity = simulation.state.download_velocity(&context);

    //
    // Save result
    //

    println!("Writing velocity output...");

    let mut writer =
        SioWriter::create("suruga_velocity.bin").expect("failed to create velocity file");

    writer
        .write_array("velocity_u", velocity.u().as_slice())
        .expect("failed to write velocity_u");

    writer
        .write_array("velocity_v", velocity.v().as_slice())
        .expect("failed to write velocity_v");

    writer
        .write_array("velocity_w", velocity.w().as_slice())
        .expect("failed to write velocity_w");

    writer.finish().expect("failed to finish velocity file");

    println!("Simulation finished.");

    Ok(())
}
