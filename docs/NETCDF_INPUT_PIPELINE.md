# Native NetCDF / GeoTIFF input pipeline

SOS processes public input data entirely in Rust. No external interpreter, virtual environment, package manager, or preprocessing scripts are required.

## Pipeline

1. `cargo run -p simulator -- get --hycom` downloads a regional NetCDF response through the public HYCOM NCSS service using `curl`.
2. `cargo run -p simulator -- generate` reads the configured terrain (NetCDF or GeoTIFF) and ocean NetCDF directly from Rust.
3. Rust validates axes and values, crops the configured geographic region, writes the terrain with `s-sof::GridWriter`, and packages ocean fields in the SOS source bundle.
4. The existing `s-sof` package builder creates `input.sof`; the simulator consumes SOF and does not read raw source files during simulation-only mode.

## Input settings

Configure `input.netcdf` in `apps/simulator/config/simulator.json`:

- `terrain_file`: `.nc`, `.tif`, or `.tiff`.
- `ocean_file`: regional HYCOM/ocean NetCDF output path.
- `prepared_dir`: Rust preprocessing output directory.
- `region`: longitude/latitude crop.
- `hycom_max_depth_m`: maximum positive-down depth.
- `hycom_url`: optional HYCOM NCSS endpoint override.
- `variables`: optional variable-name overrides for longitude, latitude, depth, time, terrain, u, v, w, temperature, and salinity.

## GeoTIFF limitations

GeoTIFF input is read through GDAL. The current implementation requires a geographic coordinate reference system, a north-up non-rotated grid, and no NoData values inside the selected region. If the file uses a projected CRS, reproject it to EPSG:4326 with a GIS tool before use. Install GDAL on macOS with `brew install gdal`.

## NetCDF limitations

The current native reader supports rectilinear one-dimensional longitude/latitude/depth axes and variables whose dimensions correspond to those axes plus optional singleton dimensions or one time dimension. Curvilinear grids and staggered velocity grids are not yet remapped. Missing/fill values are rejected rather than silently treated as measurements. Check source units and variable mapping when using a dataset other than the configured HYCOM/GEBCO products.

## Commands

```bash
cargo run -p simulator -- get --hycom
cargo run -p simulator -- get --hycom --force
cargo run -p simulator -- generate
cargo run -p simulator -- diagnose
cargo run -p simulator -- --simulation
cargo run -p simulator -- --all
```

Large public data files are not bundled with the source repository. HYCOM retrieval needs network access and a working `curl` executable. NetCDF reading is implemented with the Rust `netcdf` crate; GeoTIFF reading uses the GDAL Rust bindings.
