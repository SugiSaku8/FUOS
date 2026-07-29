use s_sof::{error::Result, package_reader::SofPackageReader};

fn main() -> Result<()> {
    let package_path = "dummy_ocean.sof";

    println!("Opening SOF package: {}", package_path);

    let reader = SofPackageReader::open(package_path)?;

    println!("Reading boundary field...");

    let ocean = reader.read_boundary()?;

    let coordinates = ocean.coordinates();

    println!("=== OceanField ===");

    println!("Grid size: {:?}", coordinates.grid().size());

    println!("Latitude points: {}", coordinates.latitude().len());

    println!("Longitude points: {}", coordinates.longitude().len());

    println!("Depth points: {}", coordinates.depth().len());

    println!("SOF package restored successfully.");

    Ok(())
}
