use anyhow::Result;
use std::fs::File;
use std::io::Read;

use s_field::{
    BoundaryField, Coordinates, Grid4D, Metadata, OceanField, SalinityField, TemperatureField,
    VelocityField,
};

use s_sof::builder::SofPackageBuilder;

fn load_bathymetry(
    path: &str,
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize,
) -> anyhow::Result<Vec<f32>> {
    let mut file = File::open(path)?;

    let mut bytes = Vec::new();

    file.read_to_end(&mut bytes)?;

    let src = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect::<Vec<_>>();

    if src.len() != src_width * src_height {
        anyhow::bail!(
            "source bathymetry size mismatch: {} != {}",
            src.len(),
            src_width * src_height
        );
    }

    let mut dst = vec![0.0; dst_width * dst_height];

    for y in 0..dst_height {
        let sy = y * src_height / dst_height;

        for x in 0..dst_width {
            let sx = x * src_width / dst_width;

            dst[y * dst_width + x] = src[sy * src_width + sx];
        }
    }

    Ok(dst)
}

fn main() -> Result<()> {
    let latitude: Vec<f64> = (0..80).map(|i| 34.5 + i as f64 * 0.01).collect();

    let longitude: Vec<f64> = (0..120).map(|i| 138.5 + i as f64 * 0.01).collect();

    let depth = vec![0.0, 10.0, 50.0, 100.0, 200.0];

    let time = vec![0.0];

    let grid = Grid4D::new(
        [time.len(), depth.len(), latitude.len(), longitude.len()],
        [1.0, 1.0, 0.01, 0.01],
    );

    println!("grid={:?}", grid.size());

    println!(
        "lat={} lon={} depth={} time={}",
        latitude.len(),
        longitude.len(),
        depth.len(),
        time.len()
    );

    let coordinates = Coordinates::new(
        grid.clone(),
        latitude.clone(),
        longitude.clone(),
        depth.clone(),
        time.clone(),
    );

    let metadata = Metadata::new();

    let velocity = VelocityField::new(grid.clone());

    let temperature = TemperatureField::new(grid.clone());

    let salinity = SalinityField::new(grid.clone());

    let boundary = BoundaryField::new(
        VelocityField::new(grid.clone()),
        VelocityField::new(grid.clone()),
        VelocityField::new(grid.clone()),
        VelocityField::new(grid.clone()),
    );

    let ocean = OceanField::new(
        coordinates,
        metadata,
        velocity,
        temperature,
        salinity,
        boundary,
    );

    //
    // bathymetry
    //
    // 現在のbin:
    // 368 × 256 = 94208
    //

    let bathymetry = load_bathymetry(
        "crates/s-builder/examples/bathymetry.bin",
        368,
        256,
        longitude.len(),
        latitude.len(),
    )?;

    println!("bathymetry={}", bathymetry.len());

    SofPackageBuilder::new("suruga_bay")
        .grid(
            grid,
            latitude.iter().map(|v| *v as f32).collect(),
            longitude.iter().map(|v| *v as f32).collect(),
            depth,
            bathymetry,
        )
        .initial(ocean.clone())
        .boundary(ocean.clone())
        .restart(ocean)
        .build("./suruga_bay.sof")?;

    println!("SOF package created.");

    Ok(())
}
