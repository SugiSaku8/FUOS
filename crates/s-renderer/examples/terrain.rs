use std::sync::Arc;

use anyhow::Result;

use winit::{event::*, event_loop::EventLoop, window::WindowAttributes};

use s_renderer::{HeightMap, Renderer, TerrainMesh, TerrainVertex};

const SHADER: &str = include_str!("../src/shader.wgsl");

#[tokio::main]
async fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window =
        Arc::new(event_loop.create_window(WindowAttributes::default().with_title("SOS Terrain"))?);

    let mut renderer = Renderer::new(window.clone(), SHADER, &[TerrainVertex::layout()]).await?;

    let heightmap = HeightMap::test();

    println!("height {:?}", heightmap.min_max());

    let terrain = TerrainMesh::from_heightmap(&heightmap, 1.0);

    let gpu_mesh = terrain.upload(renderer.device());

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
            renderer.render(&gpu_mesh).unwrap();
        }

        _ => {}
    })?;

    Ok(())
}
