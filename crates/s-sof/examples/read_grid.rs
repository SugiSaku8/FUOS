use s_sof::grid::GridReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening grid.snc...");

    let mut reader = GridReader::open("grid.snc")?;

    let grid = reader.read()?;

    println!();

    println!("Grid:");
    println!("{:?}", grid.size);

    println!();

    println!("Spacing:");
    println!("{:?}", grid.spacing);

    println!();

    println!("Coordinates:");

    println!("latitude  = {}", grid.latitude.len());

    println!("longitude = {}", grid.longitude.len());

    println!("depth     = {}", grid.depth.len());

    println!();

    println!("Bathymetry:");

    println!("samples = {}", grid.bathymetry.len());

    println!();

    println!("Grid read completed.");

    Ok(())
}
