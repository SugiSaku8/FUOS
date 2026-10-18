use s_field::Grid4D;
use s_sof::package::SofPackage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = SofPackage::open("dummy_ocean.sof")?;

    let grid = Grid4D::new([5, 2, 2, 1], [1.0, 1.0, 10.0, 1.0]);

    let latitude = vec![35.0, 35.1];

    let longitude = vec![138.5, 138.6, 138.7, 138.8, 138.9];

    let depth = vec![0.0, 50.0];

    let bathymetry = vec![-100.0; 5 * 2];

    package.write_grid(&grid, &latitude, &longitude, &depth, &bathymetry)?;

    println!("grid.snc created");

    Ok(())
}

