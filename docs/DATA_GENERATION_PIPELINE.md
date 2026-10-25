# SOS Data Generation Pipeline (Phases 1–7)

## Commands

```bash
cargo run -p simulator -- generate      # 生成単体
cargo run -p simulator -- diagnose      # 診断単体
cargo run -p simulator -- --simulation    # 既存SOFでシミュレーション単体
cargo run -p simulator -- --all           # 生成 → 診断 → シミュレーション
cargo run -p simulator -- --y             # 最初に確認し、以後のY/Nを自動でYにする
```

`--y` の最初の確認で `N` または Enter を選んだ場合は、フラグなしと同じくシミュレーション単体を実行します。フラグなしおよび `--simulation` は既存の `runs/suruga_bay/input.sof` を読み込み、入力データの生成・上書きは行いません。SOFがない場合は先に `generate` を実行してください。

## Diagnose source data first

Run the source diagnostic from the workspace root on macOS. When NetCDF inputs are configured, this command first prepares/reuses the normalized NetCDF cache and then diagnoses the derived terrain/source inputs; it does not create the final `input.sof`:

```bash
cargo run -p simulator -- diagnose
```

The diagnostic inspects the configured terrain and source bundle (or legacy source paths), checks file existence and sizes, coordinate ranges/order, terrain shape and bathymetry sign, field lengths and finite values, common physical value ranges, source/terrain geographic coverage, and whether the source depth reaches the terrain depth. It prints `PASS`, `WARN`, or `FAIL` and writes a machine-readable report to `runs/suruga_bay/diagnostics.json` (or the configured output root). Use `--json path/to/report.json` to write an additional report at a custom path.

`ERROR` findings stop generation. `WARNING` findings require confirmation in the interactive generation wizard. The explicit `diagnose` command writes a report and returns failure for errors; simulation-only mode does not run diagnostics or preprocess NetCDF because it consumes the existing SOF only. Range thresholds are conservative plausibility checks, not scientific validation. Unit conventions, source timestamps, provenance, quality flags, and missing-value semantics still need to be checked against the original provider's metadata.

## Run the interactive wizard

From the workspace root on macOS:

```bash
cargo run -p simulator -- generate
```

You can supply a different simulator config:

```bash
cargo run -p simulator -- generate path/to/simulator.json
```

When `input.netcdf` is configured, the wizard reads the configured public GEBCO/ocean NetCDF files, normalizes them, builds the terrain SNC and source bundle, diagnoses them, asks for confirmation if warnings are present, and can immediately generate the simulator input package. Legacy `coordinates.json` + `*.bin` inputs remain available only for backward compatibility. The wizard uses `apps/simulator/src/pipeline.rs` for preflight and SOF creation; the dataset-building implementation is kept in `dataset.rs`, so later Grid, interpolation, masking, initialization, and output improvements have one shared path. Simulation-only mode instead loads the existing SOF package without running generation.

## Phase mapping

1. **Interactive CLI:** `generator_cli.rs` handles prompts, validates paths, updates the config, and asks before creating the SOF package.
2. **Grid generation:** `GridBuilder` derives the solver grid from terrain depth, source latitude/longitude/depth axes, and `grid.depth_interval_m`. `grid.snc` is output, not a required solver-grid input.
3. **Input consolidation:** public NetCDF and GeoTIFF data are normalized by the native Rust `netcdf_pipeline.rs`; Rust stores coordinates and selected `u/v/w`, temperature, and salinity arrays in an `SOSSRC02` source bundle. The reader remains backward-compatible with `SOSSRC01`.
4. **Field generation:** source profiles are interpolated to solver depths and land/sub-bottom cells are masked. Velocity below the deepest source level is attenuated exponentially; it is an initialization assumption, not an observation.
5. **Initial/boundary/restart generation:** the generated `OceanField` is serialized into the initial, boundary, and restart products.
6. **SOF output:** `SofPackageBuilder` writes `grid.snc`, `terrain.snc`, `initial.snc`, `boundary.snc`, `restart.snc`, `forcing.snc`, and `manifest.json` to `runs/suruga_bay/input.sof` by default.
7. **Simulator integration:** simulation-only mode reads the existing `input.sof` package. It does not rebuild the Grid or overwrite the generated inputs.

## Single-file source bundle

The `.sosdata` layout is:

- 8-byte magic `SOSSRC02` (reader also accepts `SOSSRC01`)
- little-endian `u32` JSON-header length
- JSON header with coordinate axes, version, included-field flags, and expected sample count
- selected little-endian `f32` arrays in fixed order: `u`, `v`, optional `w`, temperature, salinity

The header records which variables are actually present. It does not pretend missing variables can be recovered from terrain. If velocity is omitted, `u/v` are initialized to zero. If temperature or salinity is omitted, the current implementation uses explicit fallback initial values of 15 °C and 35 PSU respectively and prints a warning. These are placeholders, not observations or a validated climatology. For scientific runs, include the source fields.

## Inputs and outputs

The public terrain source (`input.terrain_source`) is a NetCDF or GeoTIFF. Rust converts it to `input.terrain_output` (SNC), which the solver uses as its bathymetry grid. The scientific objective is set by `study.goal`; HYCOM variables and the default download filename are derived from that goal. The config shape is:

```json
"input": {
  "terrain_source": "data/raw/gebco_suruga.nc",
  "terrain_output": "runs/suruga_bay/preprocessed/terrain_source.snc",
  "source_bundle": "runs/suruga_bay/preprocessed/ocean_source.sosdata",
  "netcdf": {
    "prepared_dir": "runs/suruga_bay/preprocessed",
    "region": {"longitude_min": 137.5, "longitude_max": 139.5, "latitude_min": 33.5, "latitude_max": 35.5},
    "time_index": 0,
    "variables": {}
  }
}
```

`source_coordinates`, `velocity_u`, `velocity_v`, `temperature`, and `salinity` remain accepted as legacy config keys for migration. Once `source_bundle` is configured, it is authoritative; a missing bundle produces a clear error instead of silently using stale raw files.

## Public NetCDF inputs

The primary input path is GEBCO terrain NetCDF or GeoTIFF + HYCOM/ocean NetCDF → native Rust preprocessing → source bundle + `s-sof` terrain writer → `s-sof::SofPackageBuilder` → `runs/suruga_bay/input.sof`. Setup, variable mapping, supported grid assumptions, and missing-data rules are documented in [`NETCDF_INPUT_PIPELINE.md`](NETCDF_INPUT_PIPELINE.md). GeoTIFF support requires GDAL (`brew install gdal` on macOS). Large public terrain files are not bundled.

## Verification

The bundle reader validates magic, format version, coordinate monotonicity, field lengths, finite values, and trailing bytes. `source_bundle.rs` includes a round-trip unit test. Run on the target Mac:

```bash
cargo check --workspace
cargo test -p simulator
cargo test --workspace
cargo run -p simulator -- generate
cargo run -p simulator -- --simulation --cpu
```

A Rust toolchain was not available in the file-editing environment used to prepare this change, so those compilation and runtime checks still need to be run on the Mac.

## Diagnostic behavior and limitations

- Missing configured files, malformed bundle headers, invalid coordinate axes, non-finite terrain/field values, invalid field lengths, and missing paired velocity components are errors.
- Unusual but potentially valid velocities/tracers, constant fields, nonuniform horizontal axes, missing variables, source/terrain coverage mismatch, and source-depth shortfall are warnings.
- `diagnostics.json` records the status, file metadata, terrain/source summaries, field min/max/mean/zero fraction, finding codes, and remediation advice.
- This is an input sanity check, not a substitute for validating units, timestamps, source-grid staggering, observation quality, or physical-model skill. In particular, missing temperature/salinity/velocity still use the configured documented placeholder initial values if the user confirms warnings.
