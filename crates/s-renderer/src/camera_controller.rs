use winit::event::{
    ElementState,
    MouseButton,
    MouseScrollDelta,
    WindowEvent,
};

use winit::keyboard::{
    KeyCode,
    PhysicalKey,
};

use crate::camera::Camera;

use nalgebra::Vector3;



pub struct CameraController {


    speed:f32,

    sensitivity:f32,


    forward:bool,
    backward:bool,

    left:bool,
    right:bool,


    up:bool,
    down:bool,


    mouse_pressed:bool,

    last_mouse:
        Option<(f64,f64)>,



    yaw_delta:f32,

    pitch_delta:f32,


    zoom:f32,

}



impl CameraController {


    pub fn new()->Self {


        Self {


            //
            // 駿河湾スケール
            //
            speed:5000.0,


            sensitivity:0.003,


            forward:false,
            backward:false,

            left:false,
            right:false,


            up:false,
            down:false,


            mouse_pressed:false,

            last_mouse:None,


            yaw_delta:0.0,

            pitch_delta:0.0,


            zoom:0.0,

        }


    }





    pub fn process(
        &mut self,
        event:&WindowEvent
    ){



        match event {



            WindowEvent::KeyboardInput{
                event,
                ..
            }=>{


                let pressed =
                    event.state
                    ==
                    ElementState::Pressed;



                match event.physical_key {


                    PhysicalKey::Code(
                        KeyCode::KeyW
                    )=>{
                        self.forward=pressed;
                    }


                    PhysicalKey::Code(
                        KeyCode::KeyS
                    )=>{
                        self.backward=pressed;
                    }


                    PhysicalKey::Code(
                        KeyCode::KeyA
                    )=>{
                        self.left=pressed;
                    }


                    PhysicalKey::Code(
                        KeyCode::KeyD
                    )=>{
                        self.right=pressed;
                    }


                    PhysicalKey::Code(
                        KeyCode::Space
                    )=>{
                        self.up=pressed;
                    }


                    PhysicalKey::Code(
                        KeyCode::ShiftLeft
                    )=>{
                        self.down=pressed;
                    }


                    _=>{}

                }

            }





            WindowEvent::MouseInput{
                state,
                button,
                ..
            }=>{


                if *button ==
                    MouseButton::Right
                {

                    self.mouse_pressed =
                        *state
                        ==
                        ElementState::Pressed;


                    self.last_mouse=None;

                }

            }





            WindowEvent::CursorMoved{
                position,
                ..
            }=>{


                if self.mouse_pressed {


                    if let Some(
                        (x,y)
                    )=
                    self.last_mouse
                    {


                        self.yaw_delta +=
                            (position.x-x)
                            as f32
                            *
                            self.sensitivity;



                        self.pitch_delta +=
                            (position.y-y)
                            as f32
                            *
                            self.sensitivity;


                    }



                    self.last_mouse =
                        Some(
                            (
                                position.x,
                                position.y
                            )
                        );


                }


            }






            WindowEvent::MouseWheel{
                delta,
                ..
            }=>{


                let y =
                    match delta {


                        MouseScrollDelta::LineDelta(
                            _,
                            y
                        )=>*y,


                        MouseScrollDelta::PixelDelta(
                            p
                        )=>
                            p.y as f32 / 50.0,

                    };



                self.zoom += y;


            }





            _=>{}

        }


    }





    pub fn update(&mut self, camera: &mut Camera) {
        self.update_with_dt(camera, 0.016);
    }

    /// フレーム経過時間に応じてカメラを更新し、FPSの変動による移動速度差を抑える。
    pub fn update_with_dt(&mut self, camera: &mut Camera, dt_seconds: f32) {


        //
        // 回転
        //

        camera.yaw -=
            self.yaw_delta;


        camera.pitch -=
            self.pitch_delta;



        camera.pitch =
            camera.pitch.clamp(
                -1.5,
                1.5
            );



        self.yaw_delta=0.0;

        self.pitch_delta=0.0;




        //
        // ズーム
        //

        if self.zoom != 0.0 {


            let forward =
                Vector3::new(
                    camera.yaw.cos(),
                    0.0,
                    camera.yaw.sin()
                )
                .normalize();



            camera.position +=
                forward
                *
                self.zoom
                *
                10000.0;



            self.zoom=0.0;

        }







        //
        // 移動
        //

        let forward =
            Vector3::new(
                camera.yaw.cos(),
                0.0,
                camera.yaw.sin()
            )
            .normalize();



        let right =
            Vector3::new(
                -forward.z,
                0.0,
                forward.x
            );



        let mut dir =
            Vector3::zeros();



        if self.forward {
            dir+=forward;
        }


        if self.backward {
            dir-=forward;
        }


        if self.left {
            dir-=right;
        }


        if self.right {
            dir+=right;
        }


        if self.up {
            dir.y+=1.0;
        }


        if self.down {
            dir.y-=1.0;
        }





        if dir.norm()>0.0 {


            camera.position +=
                dir.normalize()
                *
                self.speed
                *
                dt_seconds.clamp(0.0, 0.05);


        }



    }


}
