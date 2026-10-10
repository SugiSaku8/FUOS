use anyhow::{Context, Result};
use std::fs;
use std::sync::Arc;

use s_field::{Coordinates, Grid4D, VelocityField};
use s_renderer::{
    FlowGenerator, FlowParticleSystem, GeoBounds, GeoConverter, GpuFlowMesh, HeightMap, Renderer,
    TerrainMesh, TerrainVertex,
};
use s_sof::grid::GridReader;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowAttributes,
};
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

    Ok(values)
}

fn load_surface_velocity() -> Result<VelocityField> {
    // 元データの格子:
    // time × depth × latitude × longitude
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

    let expected = grid.len();

    anyhow::ensure!(
        u.len() == expected,
        "u.bin のデータ数が不正です: {} != {}",
        u.len(),
        expected
    );

    anyhow::ensure!(
        v.len() == expected,
        "v.bin のデータ数が不正です: {} != {}",
        v.len(),
        expected
    );

    anyhow::ensure!(
        u.iter().chain(v.iter()).all(|x| x.is_finite()),
        "u.bin または v.bin に非有限値があります"
    );

    let mut velocity = VelocityField::with_coordinates(grid, coordinates);

    velocity.u_mut().replace(u);
    velocity.v_mut().replace(v);

    // w は今回は使用せず、ゼロのままにする。
    velocity.w_mut().fill(0.0);

    println!(
        "Ocean velocity loaded: {} horizontal grid points, {} depth levels",
        width * height,
        depth_count
    );

    Ok(velocity)
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
    // 2. SOFから地形データを読み込む
    // --------------------------------------------------

    let grid_path = "suruga_bay.sof/grid.snc";

    let mut reader = GridReader::open(grid_path)?;
    let grid = reader.read()?;

    let width = grid.longitude.len();
    let height = grid.latitude.len();

    anyhow::ensure!(
        width >= 2 && height >= 2,
        "地形データのサイズが不正です: {} x {}",
        width,
        height
    );

    anyhow::ensure!(
        grid.bathymetry.len() == width * height,
        "水深データのサイズが不正です: expected={}, actual={}",
        width * height,
        grid.bathymetry.len()
    );

    let (min_elevation, max_elevation) = grid
        .bathymetry
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), &value| {
            (min.min(value), max.max(value))
        });

    println!("=== Suruga Ocean Simulator ===");
    println!("Grid: {} x {}", width, height);
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
    // 3. 地形メッシュを生成
    // --------------------------------------------------

    let terrain_mesh = TerrainMesh::from_heightmap(&heightmap, bounds, 5.0);

    println!("Terrain vertices: {}", terrain_mesh.vertices.len());

    println!("Terrain indices: {}", terrain_mesh.indices.len());

    // --------------------------------------------------
    // 4. Rendererを初期化
    // --------------------------------------------------

    let mut renderer = Renderer::new(
        window.clone(),
        include_str!("../src/terrain.wgsl"),
        &[TerrainVertex::layout()],
    )
    .await?;

    // 地形メッシュをGPUへ転送
    let gpu_terrain = terrain_mesh.upload(renderer.device());

    // 地形と同じ地理座標変換を使う。
    let converter = GeoConverter::new(bounds, 100000.0);

    // 実データから流速場を読み込む。
    let velocity = load_surface_velocity()?;

    // 海面付近の格子点を流線の開始点にする。
    // 元データの全格子点を使用すると線が密集するため、
    // 緯度方向は2格子おきにする。
    let coordinates = velocity
        .coordinates()
        .context("海流データの座標が設定されていません")?;

    let mut seeds = Vec::new();

    for y in (0..coordinates.height()).step_by(2) {
        for x in 0..coordinates.width() {
            let lon = coordinates.longitude_at(x).unwrap();
            let lat = coordinates.latitude_at(y).unwrap();

            // 地形表示範囲内にある開始点だけを使用する。
            if lon < bounds.lon_min
                || lon > bounds.lon_max
                || lat < bounds.lat_min
                || lat > bounds.lat_max
            {
                continue;
            }

            seeds.push([lon as f32, lat as f32, 0.0]);
        }
    }

    // 900秒ずつ進め、最大24ステップ追跡する。
    // 900秒ずつ進め、最大24ステップ追跡する。
    // 粒子の流跡を最大24点保持する。
    let mut particles = FlowParticleSystem::new(&seeds, 24);

    // GPUバッファの最大頂点数。
    // 粒子1個あたり最大23線分 × 2頂点。
    let max_vertices = seeds.len() * (24 - 1) * 2;

    let mut gpu_flow = GpuFlowMesh::with_capacity(renderer.device(), max_vertices);

    // 初期状態を作る。
    particles.update(&velocity, &converter, 0.016, 120.0, 0.0);

    let initial_vertices = particles.vertices();

    if !gpu_flow.update_vertices(renderer.queue(), &initial_vertices) {
        anyhow::bail!("初期流跡がGPUバッファの容量を超えました");
    }

    println!("Flow seeds: {}", seeds.len());
    println!("Flow particles: {}", particles.particle_count());
    println!("Flow vertices: {}", initial_vertices.len());
    println!("Flow seeds: {}", seeds.len());
    println!("Flow vertices: {}", flow_vertices.len());

    let gpu_flow = if flow_vertices.is_empty() {
        None
    } else {
        Some(GpuFlowMesh::from_flow_vertices(
            renderer.device(),
            &flow_vertices,
        ))
    };

    // --------------------------------------------------
    // 5. 描画ループ
    // --------------------------------------------------

    use std::time::Instant;
　　　　let mut last_frame = Instant::now();
    event_loop.run(move |event, target| match event {
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
            if let Err(error) = renderer.render(&gpu_terrain, gpu_flow.as_ref(), None) {
                eprintln!("Render error: {error:#}");
            }
        }

        _ => {}
    })?;

    Ok(())
}
