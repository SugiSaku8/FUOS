use anyhow::Result;

use std::sync::Arc;

use s_sof::package_reader::SofPackageReader;

use s_renderer::{
    Renderer, coordinate::GeoConverter, flow_generator::FlowGenerator, flow_mesh::GpuFlowMesh,
    geo::GeoBounds, heightmap::HeightMap, mesh::GpuMesh, terrain_mesh::TerrainMesh,
};

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowAttributes,
};

fn main() -> Result<()> {
    pollster::block_on(run())
}

async fn run() -> Result<()> {
    //
    // Window
    //
    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        event_loop
            .create_window(WindowAttributes::default().with_title("Suruga Ocean Simulator"))?,
    );

    //
    // Load GEBCO
    //
    let heightmap = HeightMap::load("data/GEBCO_2025.nc")?;

    println!("GEBCO size: {} x {}", heightmap.width(), heightmap.height());

    println!(
        "Elevation range: {} m ～ {} m",
        heightmap.min(),
        heightmap.max()
    );

    //
    // Geographic bounds
    //
    let bounds = GeoBounds::new(
        heightmap.lon_min(),
        heightmap.lon_max(),
        heightmap.lat_min(),
        heightmap.lat_max(),
    );

    //
    // Coordinate converter
    //
    let converter = GeoConverter::new(bounds, 100000.0);

    //
    // Terrain
    //
    let terrain_mesh = TerrainMesh::from_heightmap(&heightmap, &converter);

    println!(
        "terrain vertices={} indices={}",
        terrain_mesh.vertices.len(),
        terrain_mesh.indices.len()
    );

    //
    // Load SOF
    //
    let package = SofPackageReader::open("data/suruga_bay.sof")?;

    let ocean = package.read_boundary()?;

    println!("OceanField loaded");

    //
    // Flow seeds
    //
    let seeds = vec![
        [heightmap.lon_min() as f32, heightmap.lat_min() as f32, 0.0],
        [heightmap.lon_max() as f32, heightmap.lat_min() as f32, 0.0],
    ];

    //
    // Generate stream lines
    //
    let flow_vertices = FlowGenerator::generate(ocean.velocity(), &seeds, 0.01, 100, 0.0);

    println!("flow vertices={}", flow_vertices.len());

    //
    // Renderer
    //
    let mut renderer = Renderer::new(
        window.clone(),
        include_str!("../src/shaders/terrain.wgsl"),
        &[TerrainMesh::vertex_layout()],
    )
    .await?;

    let gpu_terrain = GpuMesh::new(renderer.device(), &terrain_mesh);

    let gpu_flow = GpuFlowMesh::new(renderer.device(), &flow_vertices);

    //
    // Camera
    //
    renderer
        .camera_mut()
        .look_at([0.0, 5000.0, 10000.0], [0.0, 0.0, 0.0]);

    //
    // Event loop
    //
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
            renderer.update_camera();

            renderer.render(&gpu_terrain, Some(&gpu_flow)).unwrap();
        }

        _ => {}
    })?;

    Ok(())
}
