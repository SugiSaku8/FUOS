use nalgebra::{
    Matrix4,
    Perspective3,
    Point3,
    Vector3,
};



pub struct Camera {

    pub position: Point3<f32>,

    pub target: Point3<f32>,

    aspect:f32,

    fovy:f32,

    znear:f32,

    zfar:f32,

}



impl Camera {


    pub fn new(
        width:u32,
        height:u32
    )->Self{


        Self{


            //
            // 南側上空
            //
            position:
                Point3::new(
                    0.0,
                    250000.0,
                    -400000.0
                ),


            //
            // 駿河湾中心
            //
            target:
                Point3::new(
                    0.0,
                    0.0,
                    0.0
                ),


            aspect:
                width as f32 /
                height as f32,


            fovy:
                35.0_f32.to_radians(),


            znear:
                10.0,


            zfar:
                2000000.0,


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





    //
    // カメラ回転
    //
    pub fn rotate(
        &mut self,
        dx:f32,
        dy:f32
    ){


        let offset =
            self.position -
            self.target;


        let radius =
            offset.norm();



        let mut theta =
            offset.z.atan2(offset.x);


        let mut phi =
            (offset.y / radius)
            .acos();



        theta += dx * 0.005;

        phi -= dy * 0.005;



        //
        // 制限
        //
        phi =
            phi.clamp(
                0.15,
                1.45
            );



        self.position =
            Point3::new(

                radius *
                phi.sin() *
                theta.cos(),

                radius *
                phi.cos(),

                radius *
                phi.sin() *
                theta.sin()

            )
            +
            self.target.coords;


    }





    //
    // ズーム
    //
    pub fn zoom(
        &mut self,
        amount:f32
    ){


        let direction =
            self.target -
            self.position;



        let distance =
            direction.norm();



        let new_distance =
            (distance - amount)
            .clamp(
                20000.0,
                1000000.0
            );



        self.position =
            self.target -
            direction.normalize()
            *
            new_distance;


    }





    pub fn build_matrix(
        &self
    )->Matrix4<f32>{



        let view =
            Matrix4::look_at_rh(
                &self.position,
                &self.target,
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
