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

Terrain may be provided as a NetCDF grid or GeoTIFF (`.tif` / `.tiff`); ocean physical fields remain NetCDF. Set `input.netcdf.terrain_file` in `apps/simulator/config/simulator.json` to the desired terrain file.

```bash
# Download a regional HYCOM GOFS 3.1 subset into data/raw/
cargo run -p simulator -- get --hycom

# Force re-download
cargo run -p simulator -- get --hycom --force

# With terrain present but ocean input missing, --all downloads HYCOM automatically
cargo run -p simulator -- --all
```

NetCDF ingestion is implemented in Rust. GeoTIFF support requires GDAL (`brew install gdal` on macOS). See `docs/NETCDF_INPUT_PIPELINE.md` for supported formats and limits.
