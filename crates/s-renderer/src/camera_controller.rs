use winit::{
    event::{
        ElementState,
        MouseButton,
        MouseScrollDelta,
        WindowEvent,
    },
};


use crate::camera::Camera;



pub struct CameraController {


    //
    // マウス感度
    //
    pub rotate_speed:f32,


    //
    // ズーム感度
    //
    pub zoom_speed:f32,


    //
    // 左クリック状態
    //
    is_dragging:bool,


    //
    // 前回マウス位置
    //
    last_position:
        Option<(f64,f64)>,


}



impl CameraController {


    pub fn new()->Self{


        Self{


            rotate_speed:
                0.005,


            zoom_speed:
                0.1,


            is_dragging:
                false,


            last_position:
                None,


        }

    }





    pub fn process_events(
        &mut self,
        event:&WindowEvent,
        camera:&mut Camera,
    )
    -> bool{


        match event {


            //
            // マウスボタン
            //
            WindowEvent::MouseInput{
                button,
                state,
                ..
            }=>{


                if *button
                    == MouseButton::Left
                {


                    self.is_dragging =
                        *state
                        == ElementState::Pressed;



                    if !self.is_dragging {

                        self.last_position =
                            None;

                    }


                    return true;

                }


            }



            //
            // マウス移動
            //
            WindowEvent::CursorMoved{
                position,
                ..
            }=>{


                if self.is_dragging {


                    if let Some(
                        (lx,ly)
                    )
                    =
                    self.last_position
                    {


                        let dx =
                            position.x-lx;


                        let dy =
                            position.y-ly;



                        camera.rotate(

                            -(dx as f32)
                                *self.rotate_speed,


                            -(dy as f32)
                                *self.rotate_speed,

                        );

                    }


                    self.last_position =
                        Some(
                            (
                                position.x,
                                position.y
                            )
                        );


                    return true;

                }


            }



            //
            // ホイール
            //
            WindowEvent::MouseWheel{
                delta,
                ..
            }=>{


                let amount =
                    match delta {


                        MouseScrollDelta::LineDelta(
                            _,
                            y
                        )=>*y,


                        MouseScrollDelta::PixelDelta(
                            p
                        )=>p.y as f32 / 50.0,


                    };



                camera.zoom(
                    -amount*self.zoom_speed
                );


                return true;

            }



            _=>{}

        }


        false

    }

}
