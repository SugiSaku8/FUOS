use std::sync::Arc;

use anyhow::Result;

use winit::{event::*, event_loop::EventLoop, window::WindowAttributes};

use s_renderer::{Renderer, TerrainMesh, TerrainVertex};

const SHADER: &str = include_str!("../src/shader.wgsl");

fn main() -> Result<()> {
    pollster::block_on(run())
}

async fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window =
        Arc::new(event_loop.create_window(WindowAttributes::default().with_title("SOS Terrain"))?);

    let layouts = vec![TerrainVertex::layout()];

    let mut renderer = Renderer::new(window.clone(), SHADER, &layouts).await?;

    //
    // Test heightmap
    //

    let size = 128usize;

    let mut heights = Vec::with_capacity(size * size);

    for z in 0..size {
        for x in 0..size {
            let xf = x as f32 / size as f32;

            let zf = z as f32 / size as f32;

            let h = 1.0;

            heights.push(h);
        }
    }

    let terrain = TerrainMesh::from_heightmap(size, size, &heights, 1.0);

    let gpu_mesh = terrain.upload(renderer.device());

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
                    eprintln!("render error: {e:?}");
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
