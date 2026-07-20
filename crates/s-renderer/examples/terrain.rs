use anyhow::Result;

use std::sync::Arc;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowAttributes,
};

use s_renderer::{GeoBounds, HeightMap, HeightMapLoader, Renderer, TerrainMesh, TerrainVertex};

const SHADER: &str = include_str!("../src/shader.wgsl");

#[tokio::main]
async fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window = Arc::new(event_loop.create_window(
        WindowAttributes::default().with_title("SOS Terrain - GEBCO Coordinate Test"),
    )?);

    //
    // Renderer
    //

    let mut renderer = Renderer::new(window.clone(), SHADER, &[TerrainVertex::layout()]).await?;

    //
    // HeightMap
    //
    // 現在はテスト地形
    // 次段階で GEBCO Loader に置換
    //

    let heightmap = HeightMapLoader::load_gebco(
        "/Users/sugisaku/Desktop/Developer/Suruga-Wan/crates/s-renderer/examples/suruga.nc",
    )?;
    let (min, max) = heightmap.min_max();
    println!(
        "height range = {} ~ {}",
        min,
        max
    );
    //
    // 駿河湾 GEBCO 範囲
    //

    let bounds = GeoBounds {
        lon_min: 137.983333,

        lon_max: 139.022,

        lat_min: 34.583,

        lat_max: 35.425,
    };

    //
    // lon / lat → 3D terrain
    //

    let terrain = TerrainMesh::from_heightmap(&heightmap, bounds);

    println!(
        "vertices={} indices={}",
        terrain.vertices.len(),
        terrain.indices.len()
    );

    //
    // GPU Upload
    //

    let gpu_mesh = terrain.upload(renderer.device());

    //
    // Render Loop
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
                if let Err(e) = renderer.render(&gpu_mesh) {
                    eprintln!("render error: {:?}", e);
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
