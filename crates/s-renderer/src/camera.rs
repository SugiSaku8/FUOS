use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

pub struct Camera {
    pub position: Point3<f32>,

    pub target: Point3<f32>,

    pub aspect: f32,

    pub fovy: f32,

    pub znear: f32,

    pub zfar: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Point3::new(64.0, 40.0, 120.0),

            target: Point3::new(64.0, 0.0, 64.0),

            aspect: width as f32 / height as f32,

            fovy: 45.0_f32.to_radians(),

            znear: 0.1,

            zfar: 500.0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn build_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(&self.position, &self.target, &Vector3::y_axis());

        let projection =
            Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar).to_homogeneous();

        projection * view
    }
}
