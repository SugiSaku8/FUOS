use anyhow::Result;

use std::sync::Arc;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowAttributes,
};

use s_renderer::{
    CameraController, FlowGenerator, GeoBounds, GpuFlowMesh, HeightMapLoader, Renderer, TerrainMesh,
};

use s_field::{OceanGrid, generator::kuroshio::create_kuroshio};

const SHADER: &str = include_str!("../src/shader.wgsl");

fn main() -> Result<()> {
    pollster::block_on(run())
}

async fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        event_loop.create_window(
            WindowAttributes::default()
                .with_title("Suruga Ocean Terrain + Kuroshio")
                .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720)),
        )?,
    );

    let mut renderer = Renderer::new(
        window.clone(),
        SHADER,
        &[s_renderer::TerrainVertex::layout()],
    )
    .await?;

    //
    // ============================
    // Terrain
    // ============================
    //

    let bounds = GeoBounds::new(138.55, 138.95, 34.55, 35.05);

    let heightmap = HeightMapLoader::load_gebco(
        "/Users/sugisaku/Desktop/Developer/Suruga-Wan/crates/s-renderer/examples/suruga.nc",
    )?;

    println!("GEBCO size {} x {}", heightmap.width, heightmap.height);

    let terrain = TerrainMesh::from_heightmap(&heightmap, bounds);

    println!(
        "terrain vertices={} indices={}",
        terrain.vertices.len(),
        terrain.indices.len()
    );

    let gpu_mesh = terrain.upload(renderer.device());

    //
    // ============================
    // Ocean VelocityField
    // ============================
    //

    let grid = OceanGrid::suruga_bay(128, 128, 30, 1);

    let velocity = create_kuroshio(grid);

    println!("VelocityField created");

    //
    // ============================
    // Flow line
    // ============================
    //

    let seeds = vec![
        [-60.0, 0.0, 120.0],
        [-40.0, 0.0, 100.0],
        [-20.0, 0.0, 80.0],
        [0.0, 0.0, 60.0],
        [20.0, 0.0, 40.0],
    ];

    let flow_vertices = FlowGenerator::generate(&velocity, &seeds, 5.0, 100);

    println!("flow vertices={}", flow_vertices.len());

    let gpu_flow = GpuFlowMesh::new(renderer.device(), &flow_vertices);

    //
    // Camera
    //
    let mut controller = CameraController::new();

    {
        let camera = renderer.camera_mut();

        camera.position = nalgebra::Point3::new(0.0, 4000.0, 2500.0);
    }

    event_loop.run(move |event, target| match event {
        Event::WindowEvent { event, .. } => {
            controller.process(&event);

            match event {
                WindowEvent::CloseRequested => {
                    target.exit();
                }

                WindowEvent::Resized(size) => {
                    renderer.resize(size);
                }

                WindowEvent::RedrawRequested => {
                    controller.update(renderer.camera_mut());

                    renderer.render(&gpu_mesh, Some(&gpu_flow)).unwrap();
                }

                _ => {}
            }
        }

        Event::AboutToWait => {
            window.request_redraw();
        }

        _ => {}
    })?;

    Ok(())
}
