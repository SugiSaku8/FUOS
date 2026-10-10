use anyhow::{Context, Result};
use std::fs;
use std::sync::Arc;
use std::time::Instant;

use s_field::{Coordinates, Grid4D, VelocityField};

use s_renderer::{
    FlowParticleSystem, GeoBounds, GeoConverter, GpuFlowMesh, HeightMap, Renderer, TerrainMesh,
    TerrainVertex,
};

use s_sof::grid::GridReader;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowAttributes,
};

/// little-endian f32 配列としてファイルを読み込む。
fn read_f32_file(path: &str) -> Result<Vec<f32>> {
    let bytes = fs::read(path).with_context(|| format!("ファイルを読み込めません: {path}"))?;

    anyhow::ensure!(
        bytes.len() % std::mem::size_of::<f32>() == 0,
        "f32ファイルのサイズが不正です: {path}"
    );

    let values = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<_>>();

    Ok(values)
}

/// 海流の速度場を読み込む。
fn load_surface_velocity() -> Result<VelocityField> {
    let time_count = 1;
    let depth_count = 6;
    let height = 22;
    let width = 13;

    let grid = Grid4D::new(
        [time_count, depth_count, height, width],
        [1.0, 1.0, 0.04, 0.08],
    );

    let latitudes = (0..height)
        .map(|i| 34.52000045776367 + i as f64 * 0.04)
        .collect::<Vec<_>>();

    let longitudes = (0..width)
        .map(|i| 138.239990234375 + i as f64 * 0.08)
        .collect::<Vec<_>>();

    let depths = vec![0.0_f32, 10.0, 20.0, 50.0, 100.0, 200.0];

    let times = vec![0.0_f64];

    let coordinates = Coordinates::new(grid.clone(), latitudes, longitudes, depths, times);

    let u = read_f32_file("crates/s-builder/suruga_raw/u.bin")?;

    let v = read_f32_file("crates/s-builder/suruga_raw/v.bin")?;

    anyhow::ensure!(
        u.len() == grid.len(),
        "u.binの要素数が不正です: expected={}, actual={}",
        grid.len(),
        u.len()
    );

    anyhow::ensure!(
        v.len() == grid.len(),
        "v.binの要素数が不正です: expected={}, actual={}",
        grid.len(),
        v.len()
    );

    anyhow::ensure!(
        u.iter().chain(v.iter()).all(|value| value.is_finite()),
        "速度場にNaNまたは無限大が含まれています"
    );

    let mut velocity = VelocityField::with_coordinates(grid, coordinates);

    velocity.u_mut().replace(u);
    velocity.v_mut().replace(v);
    velocity.w_mut().fill(0.0);

    println!(
        "Ocean velocity loaded: {} horizontal grid points, {} depth levels",
        width * height,
        depth_count
    );

    Ok(velocity)
}

/// シミュレーターを起動する。
async fn run() -> Result<()> {
    // ------------------------------------------------------------
    // 1. ウィンドウ
    // ------------------------------------------------------------

    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        event_loop
            .create_window(WindowAttributes::default().with_title("Suruga Ocean Simulator"))?,
    );

    // ------------------------------------------------------------
    // 2. 地形データ
    // ------------------------------------------------------------

    let mut reader = GridReader::open("suruga_bay.sof/grid.snc")?;

    let grid = reader.read()?;

    // 以下は、既存のGrid型に合わせた地形情報の取得。
    let width = grid.width;
    let height = grid.height;

    let bounds = GeoBounds::new(grid.lon_min, grid.lon_max, grid.lat_min, grid.lat_max);

    println!("Terrain grid: {} x {}", width, height);

    println!("Longitude: {:.4} - {:.4}", bounds.lon_min, bounds.lon_max);

    println!("Latitude: {:.4} - {:.4}", bounds.lat_min, bounds.lat_max);

    let heightmap = HeightMap::new(width, height, grid.bathymetry);

    let terrain_mesh = TerrainMesh::from_heightmap(&heightmap, bounds, 5.0);

    // ------------------------------------------------------------
    // 3. Renderer
    // ------------------------------------------------------------

    let mut renderer = Renderer::new(
        window.clone(),
        include_str!("../src/terrain.wgsl"),
        &[TerrainVertex::layout()],
    )
    .await?;

    let gpu_terrain = terrain_mesh.upload(renderer.device());

    let converter = GeoConverter::new(bounds, 100_000.0);

    // ------------------------------------------------------------
    // 4. 海流データ
    // ------------------------------------------------------------

    let velocity = load_surface_velocity()?;

    let coordinates = velocity
        .coordinates()
        .context("速度場に座標情報がありません")?;

    // ------------------------------------------------------------
    // 5. 粒子の初期位置
    // ------------------------------------------------------------

    let mut seeds: Vec<[f32; 3]> = Vec::new();

    // 1行おきに粒子を配置する。
    for y in (0..coordinates.height()).step_by(2) {
        for x in 0..coordinates.width() {
            let Some(longitude) = coordinates.longitude_at(x) else {
                continue;
            };

            let Some(latitude) = coordinates.latitude_at(y) else {
                continue;
            };

            // 地形表示領域と重なる場所に限定する。
            if longitude < bounds.lon_min
                || longitude > bounds.lon_max
                || latitude < bounds.lat_min
                || latitude > bounds.lat_max
            {
                continue;
            }

            // 経度、緯度、深度。
            seeds.push([longitude as f32, latitude as f32, 0.0]);
        }
    }

    println!("Flow seeds: {}", seeds.len());

    // ------------------------------------------------------------
    // 6. 粒子システム
    // ------------------------------------------------------------

    const MAX_TRAIL_POINTS: usize = 24;

    let mut particles = FlowParticleSystem::new(&seeds, MAX_TRAIL_POINTS);

    // 1粒子の最大頂点数は、
    // (軌跡の点数 - 1) × 2。
    let max_vertices = seeds.len() * (MAX_TRAIL_POINTS - 1) * 2;

    let mut gpu_flow = GpuFlowMesh::with_capacity(renderer.device(), max_vertices);

    // 初期状態の軌跡を作る。
    particles.update(&velocity, &converter, 0.016, 120.0, 0.0);

    let initial_vertices = particles.vertices();

    anyhow::ensure!(
        gpu_flow.update_vertices(renderer.queue(), &initial_vertices,),
        "初期潮流線がGPUバッファの容量を超えています"
    );

    println!("Flow particles: {}", particles.particle_count());

    println!("Initial flow vertices: {}", initial_vertices.len());

    // ------------------------------------------------------------
    // 7. アニメーション開始
    // ------------------------------------------------------------

    let mut last_frame = Instant::now();

    event_loop.run(move |event, target| {
        match event {
            // ウィンドウを閉じる。
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                target.exit();
            }

            // ウィンドウサイズが変わった。
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                renderer.resize(size);
            }

            // 次のフレームを要求する。
            Event::AboutToWait => {
                window.request_redraw();
            }

            // 描画と粒子更新。
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let now = Instant::now();

                let dt_seconds = now.duration_since(last_frame).as_secs_f32();

                last_frame = now;

                // 粒子を移動させる。
                //
                // 120.0 はシミュレーション速度の倍率。
                // 実時間より速く流れを確認するための値。
                particles.update(&velocity, &converter, dt_seconds, 120.0, 0.0);

                // 現在の軌跡を頂点データに変換する。
                let flow_vertices = particles.vertices();

                // GPU上の頂点バッファを更新する。
                if !gpu_flow.update_vertices(renderer.queue(), &flow_vertices) {
                    eprintln!(
                        "Flow vertex buffer capacity exceeded: {}",
                        flow_vertices.len()
                    );
                }

                // 地形と潮流線を描画する。
                if let Err(error) = renderer.render(&gpu_terrain, Some(&gpu_flow), None) {
                    eprintln!("Render error: {error:#}");
                }
            }

            _ => {}
        }
    })?;

    Ok(())
}

fn main() -> Result<()> {
    pollster::block_on(run())
}
