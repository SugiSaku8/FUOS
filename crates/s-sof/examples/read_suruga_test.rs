use anyhow::Result;

use s_field::field::Field;
use s_sof::package_reader::SofPackageReader;

fn main() -> Result<()> {
    let path = "/Users/sugisaku/Desktop/Developer/Suruga-Wan/crates/s-sof/data/suruga_test.sof";

    println!("Opening SOF package:");
    println!("  {}", path);

    let package = SofPackageReader::open(path)?;

    println!();
    println!("=== Grid ===");

    let grid = package.read_grid()?;

    println!("Grid size:");
    println!("  {:?}", grid.size);

    println!("Spacing:");
    println!("  {:?}", grid.spacing);

    println!("Latitude:");
    println!("  {}", grid.latitude.len());

    println!("Longitude:");
    println!("  {}", grid.longitude.len());

    println!("Depth:");
    println!("  {}", grid.depth.len());

    println!("Bathymetry:");
    println!("  {}", grid.bathymetry.len());

    println!();
    println!("=== Boundary ===");

    let boundary = package.read_boundary()?;

    println!("Coordinates:");
    println!("  {:?}", boundary.coordinates());

    println!("Velocity:");

    let velocity = boundary.velocity();

    println!("  u length : {}", velocity.u().len());

    println!("  v length : {}", velocity.v().len());

    println!("  w length : {}", velocity.w().len());

    println!();
    println!("Temperature:");

    println!(
        "  size : {}",
        boundary.temperature().field().as_slice().len()
    );

    println!();
    println!("Salinity:");

    println!("  size : {}", boundary.salinity().field().as_slice().len());

    println!();
    println!("=== Field ===");

    match package.read_field() {
        Ok(field) => {
            println!("Field loaded.");

            println!("Velocity u : {}", field.velocity().u().len());

            println!("Velocity v : {}", field.velocity().v().len());

            println!("Velocity w : {}", field.velocity().w().len());

            println!(
                "Temperature : {}",
                field.temperature().field().as_slice().len()
            );

            println!("Salinity : {}", field.salinity().field().as_slice().len());
        }

        Err(e) => {
            println!("Field not found:");
            println!("  {}", e);
        }
    }

    println!();
    println!("SOF read completed.");

    Ok(())
}
