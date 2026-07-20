use anyhow::Result;

use winit::{
    event::{
        Event,
        WindowEvent,
    },

    event_loop::{
        ControlFlow,
        EventLoop,
    },

    window::Window,
};


use s_renderer::{
    create_triangle,
    GpuMesh,
    Renderer,
    TestVertex,
};



#[tokio::main]
async fn main()
    -> Result<()>
{

    env_logger::init();



    let event_loop =
        EventLoop::new()?;



    let window =
        Window::new(
            &event_loop,
        )?;



    window.set_title(
        "Suruga Ocean Simulator - Triangle Test",
    );



    let shader =
        include_str!(
            "../src/shader.wgsl"
        );



    let mut renderer =
        Renderer::new(
            &window,
            shader,
            &[
                TestVertex::layout(),
            ],
        )
        .await?;



    let mesh =
        create_triangle();



    let gpu_mesh =
        GpuMesh::upload(
            renderer
                .context()
                .device(),

            &mesh,
        );



    event_loop.run(
        move |event, target| {

            target.set_control_flow(
                ControlFlow::Poll,
            );


            match event {


                Event::WindowEvent {
                    event,
                    ..
                } => {

                    match event {


                        WindowEvent::CloseRequested =>
                        {
                            target.exit();
                        }



                        WindowEvent::Resized(
                            size
                        ) =>
                        {
                            renderer
                                .resize(size);
                        }



                        WindowEvent::RedrawRequested =>
                        {

                            match renderer.render(
                                &gpu_mesh,
                            )
                            {

                                Ok(_) =>
                                {}

                                Err(
                                    wgpu::SurfaceError::Lost
                                ) =>
                                {
                                    renderer
                                        .resize(
                                            renderer
                                                .surface_size()
                                        );
                                }


                                Err(
                                    wgpu::SurfaceError::OutOfMemory
                                ) =>
                                {
                                    target.exit();
                                }


                                Err(_) =>
                                {}

                            }
                        }


                        _ =>
                        {}

                    }
                }



                Event::AboutToWait =>
                {
                    window.request_redraw();
                }



                _ =>
                {}

            }

        },
    )?;


    Ok(())
}
