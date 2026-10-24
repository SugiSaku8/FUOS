# Suruga Ocean Simulator (SOS)

Rust-based Suruga Bay ocean simulator and data preparation pipeline.

## Generate inputs

From the workspace root, run:

```bash
cargo run -p simulator -- generate
```

The Y/N wizard packs selected source coordinates and physical fields into one `ocean_source.sosdata` bundle, updates `apps/simulator/config/simulator.json`, builds the solver Grid from the terrain and source coordinate axes, interpolates fields to the model depths, applies a sea-bottom mask, and creates the SOF package under `runs/suruga_bay/input.sof`.

Normal launch:

```bash
cargo run -p simulator
```

Details: [`docs/DATA_GENERATION_PIPELINE.md`](docs/DATA_GENERATION_PIPELINE.md).

### Streamline coverage

Streamline seeds are placed on every vertical model layer (250 m interval in the default configuration). Extra seeds are also placed at the centre of each horizontal grid cell to increase coverage while keeping the particle count lower than a full half-cell subdivision.


## 元データ診断

```bash
cargo run -p simulator -- diagnose
cargo run -p simulator -- diagnose --json reports/sos-diagnostics.json
cargo run -p simulator -- generate
```

診断は生成ウィザードと通常起動で共通の `apps/simulator/src/pipeline.rs` を通じて実行され、`ERROR` は生成を停止します。`WARNING` はY/Nウィザードで確認を求めます。詳細は `docs/DATA_GENERATION_PIPELINE.md` を参照してください。
