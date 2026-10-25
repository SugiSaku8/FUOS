# Native NetCDF / GeoTIFF input pipeline

SOS processes public input data entirely in Rust. No external interpreter, virtual environment, package manager, or preprocessing scripts are required.

## Pipeline

1. `cargo run -p simulator -- get --hycom` downloads a regional NetCDF response through the public HYCOM NCSS service using `curl`.
2. `cargo run -p simulator -- generate` reads the configured terrain (NetCDF or GeoTIFF) and ocean NetCDF directly from Rust.
3. Rust validates axes and values, crops the configured geographic region, writes the terrain with `s-sof::GridWriter`, and packages ocean fields in the SOS source bundle.
4. The existing `s-sof` package builder creates `input.sof`; the simulator consumes SOF and does not read raw source files during simulation-only mode.

## Input settings

Configure `input.netcdf` in `apps/simulator/config/simulator.json`:

- `input.terrain_source`: public terrain file (`.nc`, `.tif`, or `.tiff`).
- `input.terrain_output`: generated terrain SNC consumed by the solver.
- `study.goal`: simulation objective; determines the HYCOM variables to request.
- HYCOM is automatically saved as `data/raw/hycom_<goal>.nc`. Existing `input.netcdf.ocean_file` is accepted only as a legacy output override.
- `prepared_dir`: Rust preprocessing output directory.
- `region`: longitude/latitude crop.
- `hycom_max_depth_m`: legacy setting retained for compatibility; SOS now keeps all available depth levels and does not cap the water depth.
- `hycom_url`: optional HYCOM NCSS endpoint override.
- `variables`: optional variable-name overrides for longitude, latitude, depth, time, terrain, u, v, w, temperature, and salinity.

## GeoTIFF reprojection

GeoTIFF input is read through GDAL. Geographic north-up rasters are cropped directly. Projected rasters and rotated grids are automatically reprojected and clipped to the configured region in EPSG:4326 by `gdalwarp`. NoData values inside the selected region are rejected to avoid silently inventing bathymetry. Install GDAL (including `gdalwarp`) on macOS with `brew install gdal`.

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

## Study objective and HYCOM variable selection

Set the scientific objective in the root `study` object in `simulator.json`. HYCOM output is named automatically as `data/raw/hycom_<goal>.nc`; an old `input.netcdf.ocean_file` setting remains an optional compatibility override. Supported goals are `kuroshio_transport`, `current_field`, `temperature_distribution`, `salinity_distribution`, `heat_transport`, `water_mass`, and `full_ocean_state`. Each goal requests only the corresponding HYCOM variables. Fields not requested are omitted from the source bundle and the simulator's existing initialization fallback is used for those fields.

GeoTIFFs in projected coordinate systems or rotated grids are reprojected and clipped to the configured longitude/latitude region by GDAL `gdalwarp` to EPSG:4326 before the terrain grid is built. Install GDAL (including `gdalwarp`) with `brew install gdal`. No Python is used.

## HYCOM NCSS endpoint / 時刻の指定

`get --hycom` は NCSS Grid endpoint (`/thredds/ncss/grid/...`) を使用します。変数は NCSS の仕様に合わせて `var=water_u,water_v,...` のように1つの `var` パラメーターへまとめ、`time=last` のような NCSS が受け付けない疑似値ではなく ISO-8601 の時刻を指定します。

現在の既定データセット `GLBy0.08/expt_93.0` はアーカイブであり、公開カタログに記載された最終日付は 2024-09-04（t023 = 21:00 UTC）です。そのため、既定時刻は `2024-09-04T21:00:00Z` です。別の時刻やデータセットを使用する場合は `input.netcdf.hycom_time` と `input.netcdf.hycom_url` を設定してください。これはリアルタイム海況データではありません。

HTTP エラー時は、単なる curl の終了コードだけでなく、HTTP ステータスと NCSS 応答本文の一部をエラーメッセージに表示します。


## Download progress, timeout, and full depth

HYCOM download now shows curl's live progress bar. Connection timeout is 20 seconds and the maximum transfer time is 30 minutes; transient failures may be retried twice. The request uses `horizStride=1` and `vertStride=1`, and preprocessing retains every depth level present in the source file (no depth cap). Because the full vertical column can be large, total transfer time still depends on HYCOM server load and network speed.

## Vertical velocity when HYCOM `water_w` is unavailable

The configured `GLBy0.08/expt_93.0` NCSS dataset does not publish `water_w`, so the downloader intentionally requests only `water_u`, `water_v`, `water_temp`, and `salinity`. During preprocessing, SOS derives `w.bin` from the horizontal velocity divergence using the incompressible continuity equation:

`dw/d(depth_down) = du/dx + dv/dy`

The shallowest available HYCOM layer is assigned `w = 0` as a boundary condition and the divergence is integrated downward using the trapezoidal rule. The HYCOM packed values for `water_u`/`water_v` are decoded with their `scale_factor=0.001`; `water_temp`/`salinity` use `scale_factor=0.001` and `add_offset=20` as reported by this archive. The result is an estimate, not an observed or HYCOM-provided vertical velocity. It depends on the available horizontal velocity grid, the assumed boundary condition, and the finite-difference approximation. If a source NetCDF does provide a recognized `w` variable, the source variable is preferred.
