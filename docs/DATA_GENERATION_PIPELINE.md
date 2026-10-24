# SOS Data Generation Pipeline (Phases 1–7)

## Diagnose source data first

Run the source diagnostic from the workspace root on macOS:

```bash
cargo run -p simulator -- diagnose
```

The diagnostic inspects the configured terrain and source bundle (or legacy source paths), checks file existence and sizes, coordinate ranges/order, terrain shape and bathymetry sign, field lengths and finite values, common physical value ranges, source/terrain geographic coverage, and whether the source depth reaches the terrain depth. It prints `PASS`, `WARN`, or `FAIL` and writes a machine-readable report to `runs/suruga_bay/diagnostics.json` (or the configured output root). Use `--json path/to/report.json` to write an additional report at a custom path.

`ERROR` findings stop generation. `WARNING` findings require confirmation in the interactive wizard; normal non-interactive Simulator startup prints and saves the report but only stops on errors. Range thresholds are conservative plausibility checks, not scientific validation. Unit conventions, source timestamps, provenance, quality flags, and missing-value semantics still need to be checked against the original provider's metadata.

## Run the interactive wizard

From the workspace root on macOS:

```bash
cargo run -p simulator -- generate
```

You can supply a different simulator config:

```bash
cargo run -p simulator -- generate path/to/simulator.json
```

The wizard uses `Y/N` prompts, defaults to the supplied `crates/s-builder/suruga_raw` data, combines selected physical fields into one compact `.sosdata` file, updates the config, diagnoses the persisted inputs, asks for confirmation if warnings are present, and can immediately generate the simulator input package. Both the wizard and ordinary Simulator startup go through `apps/simulator/src/pipeline.rs`; the dataset-building implementation is kept in `dataset.rs`, so later Grid, interpolation, masking, initialization, and output improvements have one shared path.

## Phase mapping

1. **Interactive CLI:** `generator_cli.rs` handles prompts, validates paths, updates the config, and asks before creating the SOF package.
2. **Grid generation:** `GridBuilder` derives the solver grid from terrain depth, source latitude/longitude/depth axes, and `grid.depth_interval_m`. `grid.snc` is output, not a required solver-grid input.
3. **Input consolidation:** `SourceBundle` stores coordinates and selected `u/v`, temperature, and salinity arrays in one `SOSSRC01` binary container. It validates field lengths and finite values. Legacy raw files remain available as source material and can be bundled again.
4. **Field generation:** source profiles are interpolated to solver depths and land/sub-bottom cells are masked. Velocity below the deepest source level is attenuated exponentially; it is an initialization assumption, not an observation.
5. **Initial/boundary/restart generation:** the generated `OceanField` is serialized into the initial, boundary, and restart products.
6. **SOF output:** `SofPackageBuilder` writes `grid.snc`, `terrain.snc`, `initial.snc`, `boundary.snc`, `restart.snc`, `forcing.snc`, and `manifest.json` to `runs/suruga_bay/input.sof` by default.
7. **Simulator integration:** normal simulator startup reads the configured `source_bundle` and runs the same generation pipeline; it no longer needs four separate physical-field paths or a separate coordinates JSON path.

## Single-file source bundle

The `.sosdata` layout is:

- 8-byte magic `SOSSRC01`
- little-endian `u32` JSON-header length
- JSON header with coordinate axes, version, included-field flags, and expected sample count
- selected little-endian `f32` arrays in fixed order: `u`, `v`, temperature, salinity

The header records which variables are actually present. It does not pretend missing variables can be recovered from terrain. If velocity is omitted, `u/v` are initialized to zero. If temperature or salinity is omitted, the current implementation uses explicit fallback initial values of 15 °C and 35 PSU respectively and prints a warning. These are placeholders, not observations or a validated climatology. For scientific runs, include the source fields.

## Inputs and outputs

The terrain source (`terrain_source.snc`) remains a separate required input because it is both used to construct the solver depth range and retained as high-resolution rendering terrain. The preferred config now contains only:

```json
"input": {
  "terrain_source": "suruga_bay.sof/terrain_source.snc",
  "source_bundle": "crates/s-builder/suruga_raw/ocean_source.sosdata"
}
```

`source_coordinates`, `velocity_u`, `velocity_v`, `temperature`, and `salinity` remain accepted as legacy config keys for migration. Once `source_bundle` is configured, it is authoritative; a missing bundle produces a clear error instead of silently using stale raw files.

## Verification

The bundle reader validates magic, format version, coordinate monotonicity, field lengths, finite values, and trailing bytes. `source_bundle.rs` includes a round-trip unit test. Run on the target Mac:

```bash
cargo check --workspace
cargo test -p simulator
cargo test --workspace
cargo run -p simulator -- generate
cargo run -p simulator -- --cpu
```

A Rust toolchain was not available in the file-editing environment used to prepare this change, so those compilation and runtime checks still need to be run on the Mac.

## Diagnostic behavior and limitations

- Missing configured files, malformed bundle headers, invalid coordinate axes, non-finite terrain/field values, invalid field lengths, and missing paired velocity components are errors.
- Unusual but potentially valid velocities/tracers, constant fields, nonuniform horizontal axes, missing variables, source/terrain coverage mismatch, and source-depth shortfall are warnings.
- `diagnostics.json` records the status, file metadata, terrain/source summaries, field min/max/mean/zero fraction, finding codes, and remediation advice.
- This is an input sanity check, not a substitute for validating units, timestamps, source-grid staggering, observation quality, or physical-model skill. In particular, missing temperature/salinity/velocity still use the configured documented placeholder initial values if the user confirms warnings.
