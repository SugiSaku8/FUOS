use std::sync::Arc;

use anyhow::Result;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowAttributes,
};

use s_renderer::{GpuMesh, Renderer, TestVertex, create_triangle};

const VERTEX_LAYOUTS: &[wgpu::VertexBufferLayout<'static>] = &[TestVertex::layout()];

fn main() -> Result<()> {
    env_logger::init();

    pollster::block_on(run())
}

async fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window =
        Arc::new(event_loop.create_window(WindowAttributes::default().with_title("SOS Triangle"))?);

    let shader = include_str!("../src/shader.wgsl");

    let mut renderer = Renderer::new(window.clone(), shader, VERTEX_LAYOUTS).await?;

    let mesh = create_triangle();

    let gpu_mesh = GpuMesh::upload(renderer.context().device(), &mesh);

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    target.exit();
                }

                WindowEvent::Resized(size) => {
                    renderer.resize(size);
                }

                WindowEvent::RedrawRequested => {
                    let _ = renderer.render(&gpu_mesh);
                }

                _ => {}
            },

            Event::AboutToWait => {
                window.request_redraw();
            }

            _ => {}
        }
    })?;

    Ok(())
}
