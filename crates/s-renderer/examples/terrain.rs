use anyhow::Result;
use nalgebra::Point3;
use std::sync::Arc;

use winit::{
    event::*,
    event_loop::EventLoop,
    window::{
        WindowAttributes
    },
};


use s_renderer::{
    GeoBounds,
    HeightMapLoader,
    Renderer,
    TerrainMesh,
    TerrainVertex,
};



const SHADER:&str =
include_str!("../src/shader.wgsl");




fn main()->Result<()> {


    env_logger::init();


    let event_loop =
        EventLoop::new()?;


    let window =
        Arc::new(
            event_loop.create_window(
                WindowAttributes::default()
                .with_title(
                    "Suruga Ocean Terrain"
                )
                .with_inner_size(
                    winit::dpi::LogicalSize::new(
                        1280,
                        720
                    )
                )
            )?
        );



    let mut renderer =
        pollster::block_on(
            Renderer::new(
                window.clone(),
                SHADER,
                &[
                    TerrainVertex::layout()
                ],
            )
        )?;



    //
    // 駿河湾範囲
    //
    let bounds =
        GeoBounds::new(
            138.55,
            138.95,
            34.8,
            35.15,
        );



    let heightmap =
        HeightMapLoader::load_gebco(
            "/Users/sugisaku/Desktop/Developer/Suruga-Wan/crates/s-renderer/examples/suruga.nc"
        )?;



    println!(
        "GEBCO size: {} x {}",
        heightmap.width,
        heightmap.height
    );



    let terrain =
        TerrainMesh::from_heightmap(
            &heightmap,
            bounds
        );



    let gpu_mesh =
        terrain.upload(
            renderer.device()
        );



    //
    // 南側から見る
    //
    renderer.camera_mut().position =
        Point3::new(
            0.0,
            2500.0,
            -5000.0
        );


    renderer.camera.target =
        nalgebra::Point3::new(
            0.0,
            0.0,
            0.0,
        );




    event_loop.run(
        move |event,target|
        {

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



                        WindowEvent::Resized(size)=>
                        {
                            renderer.resize(size);
                        }



                        WindowEvent::RedrawRequested =>
                        {

                            renderer
                                .render(
                                    &gpu_mesh
                                )
                                .unwrap();

                        }


                        _=>{}

                    }

                }



                Event::AboutToWait =>
                {
                    window.request_redraw();
                }



                _=>{}

            }

        }
    )?;


    Ok(())
}
