use nalgebra::{
    Matrix4,
    Perspective3,
    Point3,
    Vector3,
};


pub struct Camera {

    pub position: Point3<f32>,

    pub yaw: f32,

    pub pitch: f32,


    pub aspect: f32,

    pub fovy: f32,

    pub znear: f32,

    pub zfar: f32,
}



impl Camera {


    pub fn new(
        width:u32,
        height:u32
    )->Self {


        Self {

            //
            // 南側上空から北を見る
            //
            position:
                Point3::new(
                    0.0,
                    20000.0,
                    30000.0
                ),


            yaw:
                (-90.0_f32)
                .to_radians(),


            pitch:
                (-35.0_f32)
                .to_radians(),


            aspect:
                width as f32 /
                height as f32,


            fovy:
                45.0_f32.to_radians(),


            znear:
                1.0,


            //
            // 駿河湾スケール
            //
            zfar:
                200000.0,

        }

    }



    pub fn resize(
        &mut self,
        width:u32,
        height:u32
    ){

        self.aspect =
            width as f32 /
            height as f32;

    }



    pub fn build_matrix(
        &self
    )->Matrix4<f32>{


        let direction =
            Vector3::new(

                self.yaw.cos()
                *
                self.pitch.cos(),


                self.pitch.sin(),


                self.yaw.sin()
                *
                self.pitch.cos()

            )
            .normalize();



        let target =
            self.position
            +
            direction;



        let view =
            Matrix4::look_at_rh(
                &self.position,
                &target,
                &Vector3::y_axis()
            );



        let projection =
            Perspective3::new(
                self.aspect,
                self.fovy,
                self.znear,
                self.zfar
            )
            .to_homogeneous();



        projection * view

    }


}
