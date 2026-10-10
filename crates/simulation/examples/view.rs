use anyhow::Result;

use std::sync::Arc;

use winit::{event::*, event_loop::EventLoop, window::WindowAttributes};

use s_renderer::{HeightMapLoader, Renderer, TerrainMesh, TerrainVertex};

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window = event_loop
        .create_window(WindowAttributes::default().with_title("Suruga Ocean Simulator"))?;

    let window = Arc::new(window);

    //
    // =========================
    // Load SOF terrain
    // =========================
    //

    let heightmap = HeightMapLoader::load_test();

    println!(
        "HeightMap loaded: {} x {}",
        heightmap.width, heightmap.height
    );

    //
    // Terrain bounds
    //
    // SOFの場合は既存座標に合わせる
    //

    let bounds = s_renderer::GeoBounds {
        lat_min: 34.5,

        lat_max: 35.2,

        lon_min: 138.2,

        lon_max: 139.0,
    };

    //
    // Build terrain mesh
    //

    let terrain = TerrainMesh::from_heightmap(&heightmap, bounds, 0.02);

    println!("Terrain vertices : {}", terrain.vertices.len());

    println!("Terrain indices : {}", terrain.indices.len());

    //
    // Renderer
    //

    let shader = include_str!("../../s-renderer/src/shader.wgsl");

    let mut renderer = pollster::block_on(Renderer::new(
        window.clone(),
        shader,
        &[TerrainVertex::layout()],
    ))?;

    //
    // Upload GPU mesh
    //

    let gpu_terrain = terrain.upload(renderer.device());

    //
    // Render loop
    //

    event_loop.run(move |event, target| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                target.exit();
            }

            WindowEvent::Resized(size) => {
                renderer.resize(size);
            }

            WindowEvent::RedrawRequested => {
                if let Err(e) = renderer.render(&gpu_terrain, None, None) {
                    eprintln!("Render error: {}", e);
                }
            }

            _ => {}
        },

        Event::AboutToWait => {
            window.request_redraw();
        }

        _ => {}
    })?;

    Ok(())
}
