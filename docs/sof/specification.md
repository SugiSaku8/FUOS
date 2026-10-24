# Suruga Ocean Format (SOF) Specification

**Version:** 0.1.0 (Draft)  
**Format Name:** Suruga Ocean Format (SOF)  
**Data Container:** NetCDF4 (HDF5)  
**Primary Metadata Format:** Apple Pkl

---

# 1. 概要

Suruga Ocean Format（SOF）は、Suruga Ocean Simulator専用に設計された海洋データフォーマットである。

SOFは

- Rustによる公開データ取得・前処理・SOF生成・シミュレーション
- GPUによる可視化

の共通フォーマットとして利用する。

SOFはシミュレーションエンジンから外部データ形式を完全に分離することを目的とする。

---

# 2. 設計思想

SOFは以下の原則に従う。

- シンプル
- 高速
- 拡張可能
- 人間が理解しやすい
- 外部データ形式とシミュレーションを分離する
- 特定の海洋モデル（ROMS・MITgcm等）に依存しない

---

# 3. ファイル構成

SOFは「パッケージ」を単位として管理する。

```
example.sof/

    manifest.pkl

    grid.snc

    boundary.snc

    restart.snc

    preview.png
```

---

# 4. 拡張子

|拡張子|用途|
|-------|----|
|.sof|SOFパッケージ|
|.snc|Suruga NetCDF|
|.pkl|メタデータ・設定|
|.png|プレビュー画像（任意）|

---

# 5. パッケージ構造

## manifest.pkl

パッケージ全体を管理する。

例

```pkl
format = "SOF"

version = "0.1.0"

title = "Suruga Bay"

creator = "Suruga Ocean"

created = "2026-07-20"

grid = "grid.snc"

boundary = "boundary.snc"

restart = "restart.snc"
```

---

# 6. データファイル

## grid.snc

海底地形・格子情報

### 必須変数

|名前|説明|
|----|----|
|x|東方向座標|
|y|北方向座標|
|z|鉛直座標|
|depth|海底深度|
|mask|海陸マスク|
|dx|セル幅|
|dy|セル幅|
|dz|セル高さ|

---

## boundary.snc

境界条件

### 必須変数

|名前|単位|
|----|----|
|u|m/s|
|v|m/s|
|w|m/s|
|temperature|℃|
|salinity|PSU|
|density|kg/m³（任意）|

---

## restart.snc

再開用データ

保存対象

- velocity
- temperature
- salinity
- density
- simulation_time

---

# 7. 座標系

SOFは右手座標系を採用する。

```
+x 東

+y 北

+z 上
```

海面

```
z = 0
```

海中

```
z < 0
```

---

# 8. 単位

SOFはSI単位系を使用する。

|物理量|単位|
|------|----|
|長さ|m|
|時間|s|
|速度|m/s|
|密度|kg/m³|
|温度|℃|
|塩分|PSU|

---

# 9. 時間

時間はUnix Timeで保持する。

```
seconds since 1970-01-01 00:00:00 UTC
```

変数名

```
time
```

---

# 10. 命名規則

## 変数

すべて

- 小文字
- snake_case

例

```
temperature

salinity

sea_surface_height

bathymetry
```

---

## クレート

大型モジュール

```
s-core

s-grid

s-io

s-physics

s-solver

s-renderer

s-gui
```

---

## Rustモジュール

```
bathymetry.rs

boundary.rs

reader.rs

grid.rs

geometry.rs
```

---

## Struct

PascalCase

```
OceanGrid

BoundaryCondition

Bathymetry
```

---

## 関数

snake_case

```
load_grid()

update_density()

compute_pressure()
```

---

# 11. データ生成の役割

Rustが公開NetCDF / GeoTIFFの読み込み、検査、前処理を行い、`s-sof` を通じてSOFパッケージを生成する。

---

# 13. データフロー

```
HYCOM / GEBCO

↓

Rust input pipeline + s-sof

↓

SOF

↓

Rust

↓

Simulation

↓

GPU

↓

Viewer
```

---

# 14. 将来追加予定

SOFは以下の拡張を想定する。

|ファイル|用途|
|---------|----|
|forcing.snc|風・熱フラックス|
|tide.snc|潮汐|
|wave.snc|波浪|
|river.snc|河川|
|particle.snc|粒子|
|sediment.snc|土砂|
|biology.snc|生態系|

---

# 15. バージョン管理

Semantic Versioningを採用する。

```
Major.Minor.Patch
```

例

```
0.1.0

0.2.0

1.0.0
```

Majorが異なる場合、互換性は保証しない。

---

# 16. 今後の方針

SOF v0.1は、黒潮の駿河湾流入シミュレーション（First Step）を対象とした最小仕様である。

今後は潮汐・波浪・河川・気象・粒子・土砂・生態系を追加し、研究用途にも利用できる汎用的な海洋データフォーマットへ発展させる。
