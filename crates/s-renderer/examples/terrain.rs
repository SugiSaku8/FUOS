use anyhow::{Context, Result};
use s_field::{Coordinates, Grid4D, VelocityField};
use s_renderer::{
    GeoBounds, GeoConverter, GpuFlowMesh, HeightMap, Renderer, TerrainMesh, TerrainVertex,
    camera_controller::CameraController, contour::generate_bathymetry_contours,
    flow_generator::FlowParticleSystem, terrain::OceanSurfaceMesh,
};
use s_sof::grid::GridReader;
use std::{fs, sync::Arc, time::Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowAttributes,
};

const GRID_PATH: &str = "suruga_bay.sof/grid.snc";
const RAW_DATA_DIR: &str = "crates/s-builder/suruga_raw";

const SOURCE_DEPTH_COUNT: usize = 6;
const DEPTH_INTERVAL_M: usize = 250;
const GRID_HEIGHT: usize = 22;
const GRID_WIDTH: usize = 13;

const PARTICLE_TRAIL_POINTS: usize = 24;
const PARTICLE_SPEED_MULTIPLIER: f32 = 120.0;

/// 海流の移動速度を確認しながら、f32のバイナリデータを読み込む。
fn read_f32_file(path: &str) -> Result<Vec<f32>> {
    let bytes = fs::read(path).with_context(|| format!("ファイルを読み込めません: {path}"))?;

    anyhow::ensure!(
        bytes.len() % 4 == 0,
        "f32ファイルのサイズが4の倍数ではありません: {path}"
    );

    let values = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<_>>();

    anyhow::ensure!(
        values.iter().all(|value| value.is_finite()),
        "{path} に非有限値があります"
    );

    Ok(values)
}

/// バイナリデータから海流の速度場を読み込む。
fn load_surface_velocity(max_seabed_depth_m: f32) -> Result<VelocityField> {
    // 元のシミュレーション出力は0〜200mの6層。
    // 海底の最大水深まで250m間隔の描画層を作る。
    // 200mより深い層は200m層を基準に深度減衰させる簡易モデル。
    // 配列の順序: time × depth × latitude × longitude
    let max_sim_depth_m = (((max_seabed_depth_m.max(0.0) as usize)
        + DEPTH_INTERVAL_M - 1)
        / DEPTH_INTERVAL_M)
        * DEPTH_INTERVAL_M;
    let depth_count = max_sim_depth_m / DEPTH_INTERVAL_M + 1;
    let grid = Grid4D::new(
        [1, depth_count, GRID_HEIGHT, GRID_WIDTH],
        [1.0, DEPTH_INTERVAL_M as f32, 0.04, 0.08],
    );

    let latitudes = (0..GRID_HEIGHT)
        .map(|i| 34.52000045776367 + i as f64 * 0.04)
        .collect::<Vec<_>>();

    let longitudes = (0..GRID_WIDTH)
        .map(|i| 138.239990234375 + i as f64 * 0.08)
        .collect::<Vec<_>>();

    let depths = (0..depth_count)
        .map(|i| (i * DEPTH_INTERVAL_M) as f32)
        .collect::<Vec<_>>();
    let times = vec![0.0_f64];

    let coordinates = Coordinates::new(grid.clone(), latitudes, longitudes, depths, times);

    let u_path = format!("{RAW_DATA_DIR}/u.bin");
    let v_path = format!("{RAW_DATA_DIR}/v.bin");

    let u = read_f32_file(&u_path)?;
    let v = read_f32_file(&v_path)?;

    let source_expected = SOURCE_DEPTH_COUNT * GRID_HEIGHT * GRID_WIDTH;

    anyhow::ensure!(
        u.len() == source_expected,
        "u.bin のデータ数が不正です: {} != {}",
        u.len(),
        source_expected
    );
    anyhow::ensure!(
        v.len() == source_expected,
        "v.bin のデータ数が不正です: {} != {}",
        v.len(),
        source_expected
    );

    // シミュレーション出力を250m間隔の描画用格子へ写す。
    // 200mより深い層は、200m層の速度を指数減衰させて簡易的に拡張する。
    // これは未計算の深部を実測値として扱うものではなく、深度方向の簡易モデル。
    let horizontal_count = GRID_HEIGHT * GRID_WIDTH;
    let source_depths = [0.0_f32, 10.0, 20.0, 50.0, 100.0, 200.0];
    let mut expanded_u = Vec::with_capacity(grid.len());
    let mut expanded_v = Vec::with_capacity(grid.len());

    for &depth in coordinates.depth() {
        if depth <= source_depths[0] {
            expanded_u.extend_from_slice(&u[..horizontal_count]);
            expanded_v.extend_from_slice(&v[..horizontal_count]);
            continue;
        }

        if depth <= source_depths[SOURCE_DEPTH_COUNT - 1] {
            let upper = source_depths
                .iter()
                .position(|&source_depth| source_depth >= depth)
                .unwrap_or(SOURCE_DEPTH_COUNT - 1);
            let lower = upper.saturating_sub(1);
            let span = source_depths[upper] - source_depths[lower];
            let fraction = if span > 0.0 {
                (depth - source_depths[lower]) / span
            } else {
                0.0
            };
            let lower_offset = lower * horizontal_count;
            let upper_offset = upper * horizontal_count;
            for i in 0..horizontal_count {
                expanded_u.push(
                    u[lower_offset + i] * (1.0 - fraction)
                        + u[upper_offset + i] * fraction,
                );
                expanded_v.push(
                    v[lower_offset + i] * (1.0 - fraction)
                        + v[upper_offset + i] * fraction,
                );
            }
            continue;
        }

        let last_offset = (SOURCE_DEPTH_COUNT - 1) * horizontal_count;
        let attenuation = (-(depth - source_depths[SOURCE_DEPTH_COUNT - 1]) / 1200.0).exp();
        for i in 0..horizontal_count {
            expanded_u.push(u[last_offset + i] * attenuation);
            expanded_v.push(v[last_offset + i] * attenuation);
        }
    }

    let mut velocity = VelocityField::with_coordinates(grid, coordinates);
    velocity.u_mut().replace(expanded_u);
    velocity.v_mut().replace(expanded_v);
    // 入力データに鉛直流速wがないため、根拠のない一定の鉛直流を加えない。
    velocity.w_mut().fill(0.0);
    println!(
        "Ocean velocity loaded: {} x {} grid, {} depth levels ({}m interval, 0-{}m)",
        GRID_WIDTH, GRID_HEIGHT, depth_count, DEPTH_INTERVAL_M, max_sim_depth_m
    );

    // 速度データの統計を表示する。
    //
    // Fieldのデータ取得APIはプロジェクトの実装に依存するため、
    // ここでは読み込んだバイナリを直接再利用して統計を計算する。
    let u = read_f32_file(&u_path)?;
    let v = read_f32_file(&v_path)?;

    let mut min_speed = f32::INFINITY;
    let mut max_speed = 0.0_f32;
    let mut moving_points = 0usize;

    for (&u_value, &v_value) in u.iter().zip(v.iter()) {
        let speed = u_value.hypot(v_value);

        min_speed = min_speed.min(speed);
        max_speed = max_speed.max(speed);

        if speed >= 0.02 {
            moving_points += 1;
        }
    }

    println!(
        "Current speed range: {:.4} - {:.4} m/s",
        min_speed, max_speed
    );
    println!("Moving grid points: {}", moving_points);

    Ok(velocity)
}

/// 海流の開始位置を格子から生成する。
fn generate_seeds(
    coordinates: &Coordinates,
    bounds: &GeoBounds,
    heightmap: &HeightMap,
) -> Vec<[f32; 3]> {
    let mut seeds = Vec::new();

    for &depth in coordinates.depth() {
        for y in (0..coordinates.height()).step_by(2) {
            for x in 0..coordinates.width() {
                let Some(lon) = coordinates.longitude_at(x) else {
                    continue;
                };
                let Some(lat) = coordinates.latitude_at(y) else {
                    continue;
                };

                if lon < bounds.lon_min
                    || lon > bounds.lon_max
                    || lat < bounds.lat_min
                    || lat > bounds.lat_max
                {
                    continue;
                }

                // 海底より下に粒子を置かない。地形格子上の最近傍点で水深を確認する。
                let map_x = (((lon - bounds.lon_min) / (bounds.lon_max - bounds.lon_min))
                    * (heightmap.width - 1) as f64)
                    .round() as usize;
                let map_y = (((lat - bounds.lat_min) / (bounds.lat_max - bounds.lat_min))
                    * (heightmap.height - 1) as f64)
                    .round() as usize;
                let map_x = map_x.min(heightmap.width - 1);
                let map_y = map_y.min(heightmap.height - 1);
                let elevation = heightmap.get(map_x, map_y);
                let seabed_depth = if elevation < 0.0 { -elevation } else { 0.0 };

                if elevation < 0.0 && depth <= seabed_depth {
                    seeds.push([lon as f32, lat as f32, depth]);
                }
            }
        }
    }

    seeds
}

fn main() -> Result<()> {
    pollster::block_on(run())
}

async fn run() -> Result<()> {
    // --------------------------------------------------
    // 1. ウィンドウ
    // --------------------------------------------------

    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        event_loop
            .create_window(WindowAttributes::default().with_title("Suruga Ocean Simulator"))?,
    );

    // --------------------------------------------------
    // 2. 地形データの読み込み
    // --------------------------------------------------

    let mut reader = GridReader::open(GRID_PATH)?;
    let grid = reader.read()?;

    let width = grid.longitude.len();
    let height = grid.latitude.len();

    anyhow::ensure!(
        width >= 2 && height >= 2,
        "地形格子のサイズが不正です: {} x {}",
        width,
        height
    );

    anyhow::ensure!(
        grid.bathymetry.len() == width * height,
        "水深データのサイズが不正です: expected={}, actual={}",
        width * height,
        grid.bathymetry.len()
    );

    anyhow::ensure!(
        grid.longitude.iter().all(|v| v.is_finite())
            && grid.latitude.iter().all(|v| v.is_finite())
            && grid.bathymetry.iter().all(|v| v.is_finite()),
        "地形データに非有限値があります"
    );

    let (min_elevation, max_elevation) = grid
        .bathymetry
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), &value| {
            (min.min(value), max.max(value))
        });

    println!("=== Suruga Ocean Simulator ===");
    println!("Terrain grid: {} x {}", width, height);
    println!(
        "Longitude: {:.4} - {:.4}",
        grid.longitude[0],
        grid.longitude[width - 1]
    );
    println!(
        "Latitude: {:.4} - {:.4}",
        grid.latitude[0],
        grid.latitude[height - 1]
    );
    println!(
        "Bathymetry: {:.2} m - {:.2} m",
        min_elevation, max_elevation
    );

    let bounds = GeoBounds::new(
        grid.longitude[0] as f64,
        grid.longitude[width - 1] as f64,
        grid.latitude[0] as f64,
        grid.latitude[height - 1] as f64,
    );

    let heightmap = HeightMap::new(width, height, grid.bathymetry);

    // --------------------------------------------------
    // 3. 地形メッシュ
    // --------------------------------------------------

    let terrain_mesh = TerrainMesh::from_heightmap(&heightmap, bounds, 5.0);
    let ocean_mesh = OceanSurfaceMesh::new(bounds, &heightmap);
    println!("Terrain vertices: {}", terrain_mesh.vertex_count());
    println!("Terrain indices: {}", terrain_mesh.index_count());

    // --------------------------------------------------
    // 4. Renderer
    // --------------------------------------------------

    let mut renderer = Renderer::new(
        window.clone(),
        include_str!("../src/terrain.wgsl"),
        &[TerrainVertex::layout()],
    )
    .await?;

    let gpu_terrain = terrain_mesh.upload(renderer.device());
    let gpu_ocean = ocean_mesh.upload(renderer.device());

    // --------------------------------------------------
    // 4.1 海底等深線
    // --------------------------------------------------

    const CONTOUR_INTERVAL_M: f32 = 250.0;
    const TERRAIN_VERTICAL_SCALE: f32 = 5.0;

    let contour_vertices = generate_bathymetry_contours(
        &heightmap,
        bounds,
        TERRAIN_VERTICAL_SCALE,
        CONTOUR_INTERVAL_M,
    );

    let gpu_contours = GpuFlowMesh::from_flow_vertices(renderer.device(), &contour_vertices);

    println!(
        "Bathymetry contours: {} vertices ({} m interval)",
        contour_vertices.len(),
        CONTOUR_INTERVAL_M
    );

    let converter = GeoConverter::new(bounds, 100_000.0);

    // --------------------------------------------------
    // 5. 海流データと粒子
    // --------------------------------------------------

    let max_seabed_depth_m = heightmap
        .data
        .iter()
        .filter(|&&elevation| elevation < 0.0)
        .map(|&elevation| -elevation)
        .fold(0.0_f32, f32::max);
    let velocity = load_surface_velocity(max_seabed_depth_m)?;

    let coordinates = velocity
        .coordinates()
        .context("海流データの座標が設定されていません")?;

    let seeds = generate_seeds(coordinates, &bounds, &heightmap);

    anyhow::ensure!(
        !seeds.is_empty(),
        "地形範囲内に海流粒子の開始位置がありません"
    );

    let mut particles = FlowParticleSystem::new(&seeds, PARTICLE_TRAIL_POINTS);

    let max_vertices = seeds.len() * ((PARTICLE_TRAIL_POINTS - 1) * 2 + 6);

    let mut gpu_flow = GpuFlowMesh::with_capacity(renderer.device(), max_vertices);
    let mut gpu_particle_dots = GpuFlowMesh::with_capacity(renderer.device(), seeds.len());
    let mut camera_controller = CameraController::new();

    println!("Flow seeds: {}", seeds.len());
    println!("Flow particles: {}", particles.particle_count());
    println!("Flow vertex buffer capacity: {}", max_vertices);

    // --------------------------------------------------
    // 6. 描画・時間発展ループ
    // --------------------------------------------------

    let mut last_frame = Instant::now();
    let mut frame_count = 0_u64;
    let mut diagnostic_elapsed = 0.0_f32;

    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                target.exit();
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                renderer.resize(size);
            }

            Event::AboutToWait => {
                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let now = Instant::now();
                let dt_seconds = now.duration_since(last_frame).as_secs_f32();
                last_frame = now;

                // 毎フレーム、海流粒子を更新する。
                particles.update(
                    &velocity,
                    &converter,
                    dt_seconds,
                    PARTICLE_SPEED_MULTIPLIER,
                    0.0,
                );

                let flow_vertices = particles.vertices();

                // 軌跡と粒子マーカーをGPUへ転送する。
                if !gpu_flow.update_vertices(renderer.queue(), &flow_vertices) {
                    eprintln!(
                        "Flow vertex buffer capacity exceeded: {} vertices",
                        flow_vertices.len()
                    );
                }

                // vertices() に粒子マーカーも含まれるため、
                // 独立した粒子用バッファへの更新は行わない。

                camera_controller.update_with_dt(renderer.camera_mut(), dt_seconds);

                // 一定間隔で診断情報を出力する。
                diagnostic_elapsed += dt_seconds;
                frame_count += 1;

                if diagnostic_elapsed >= 2.0 {
                    println!(
                        "FPS: {:.1}, flow vertices: {}",
                        frame_count as f32 / diagnostic_elapsed,
                        flow_vertices.len()
                    );

                    diagnostic_elapsed = 0.0;
                    frame_count = 0;
                }

                // 地形と流線を描画する。
                if let Err(error) = renderer.render_with_ocean_and_particles(
                    &gpu_terrain,
                    Some(&gpu_ocean),
                    Some(&gpu_contours),
                    Some(&gpu_flow),
                    Some(&gpu_particle_dots),
                    None,
                ) {
                    eprintln!("Render error: {error:#}");
                }
            }

            Event::WindowEvent { event, .. } => {
                camera_controller.process(&event);
            }

            _ => {}
        }
    })?;

    Ok(())
}
