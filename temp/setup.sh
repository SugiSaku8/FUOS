# =========================
# プロジェクト作成
# =========================

git init

# Workspace用ディレクトリ
mkdir crates
mkdir apps
mkdir assets
mkdir docs
mkdir tests

# assets
mkdir -p assets/gebco
mkdir -p assets/hycom
mkdir -p assets/cache
mkdir -p assets/shaders
mkdir -p assets/textures

# シミュレーション設定
mkdir -p simulation/boundary
mkdir -p simulation/restart
mkdir -p simulation/config

# =========================
# Rust Crates
# =========================

cargo new crates/core --lib
cargo new crates/grid --lib
cargo new crates/physics --lib
cargo new crates/solver --lib
cargo new crates/io --lib
cargo new crates/math --lib
cargo new crates/renderer --lib
cargo new crates/gui --lib
cargo new crates/util --lib

# 実行アプリ
cargo new apps/simulator

# =========================
# Git
# =========================

touch .gitignore
touch README.md
touch Cargo.toml
