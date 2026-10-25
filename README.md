# Suruga Ocean Simulator (SOS)

Rust-based Suruga Bay ocean simulator and data preparation pipeline.

## Commands

From the workspace root:

```bash
# Generate/update source bundle and create input.sof (interactive)
cargo run -p simulator -- generate

# Diagnose inputs only
cargo run -p simulator -- diagnose

# Run simulation only from an existing input.sof
cargo run -p simulator -- --simulation

# Run generate → diagnose → simulation
cargo run -p simulator -- --all

# Confirm once, then automatically answer Y to generator prompts
cargo run -p simulator -- --y
```

`--y` asks for an initial confirmation. If you answer `N` (or press Enter), it behaves like the default command and runs simulation only. The simulation-only command does not generate or overwrite `input.sof`; run `generate` first if it does not exist.

Details: [`docs/DATA_GENERATION_PIPELINE.md`](docs/DATA_GENERATION_PIPELINE.md).

### Public NetCDF input setup

The primary data path is public GEBCO terrain NetCDF or GeoTIFF + HYCOM/ocean NetCDF → native Rust preprocessing → `s-sof` SOF package. Install GDAL on macOS for GeoTIFF support (`brew install gdal`), then set paths in `apps/simulator/config/simulator.json`. HYCOM can be retrieved with `cargo run -p simulator -- get --hycom`.

Variable aliases, region selection, missing-value policy, and supported grid types are documented in [`docs/NETCDF_INPUT_PIPELINE.md`](docs/NETCDF_INPUT_PIPELINE.md).

### Streamline coverage

Streamline seeds are placed on every vertical model layer (250 m interval in the default configuration). Extra seeds are also placed at the centre of each horizontal grid cell to increase coverage while keeping the particle count lower than a full half-cell subdivision.


## 元データ診断

```bash
cargo run -p simulator -- diagnose
cargo run -p simulator -- diagnose --json reports/sos-diagnostics.json
cargo run -p simulator -- generate
```

診断単体は `apps/simulator/src/pipeline.rs` を通じて実行され、`ERROR` は診断コマンドを失敗として終了します。`WARNING` はY/Nウィザードで確認を求めます。詳細は `docs/DATA_GENERATION_PIPELINE.md` を参照してください。


### Public input data (NetCDF / GeoTIFF)

Terrain may be provided as a NetCDF grid or GeoTIFF (`.tif` / `.tiff`); ocean physical fields remain NetCDF. Set `input.terrain_source` in `apps/simulator/config/simulator.json` to the desired terrain file.

```bash
# Download a regional HYCOM GOFS 3.1 subset into data/raw/
cargo run -p simulator -- get --hycom

# Force re-download
cargo run -p simulator -- get --hycom --force

The command parser treats `get`, `generate`, and `diagnose` as subcommands (not as config-file paths). An optional config path goes after the command, for example `cargo run -p simulator -- get --hycom apps/simulator/config/simulator.json --force`. The shared horizontal study window is `input.netcdf.region`; `get` reports the actual HYCOM coordinate-centre extent, and `generate` clips the rendered terrain to that same actual HYCOM coverage before building the solver grid.

# With terrain present but ocean input missing, --all downloads HYCOM automatically
cargo run -p simulator -- --all
```

NetCDF ingestion is implemented in Rust. GeoTIFF support requires GDAL (`brew install gdal` on macOS). See `docs/NETCDF_INPUT_PIPELINE.md` for supported formats and limits.

### Study objective selects HYCOM data

Set `study.goal` in `apps/simulator/config/simulator.json` instead of configuring a specific ocean input filename. The downloader derives `data/raw/hycom_<goal>.nc` and requests only the variables needed by that objective. Supported values: `kuroshio_transport`, `current_field`, `temperature_distribution`, `salinity_distribution`, `heat_transport`, `water_mass`, `full_ocean_state`. `input.terrain_source` selects the public NetCDF/GeoTIFF terrain; `input.terrain_output` is the generated SNC grid.

Projected/rotated GeoTIFFs are automatically reprojected and clipped to the configured region using GDAL `gdalwarp` (included with `brew install gdal`). Python is fully excluded.
