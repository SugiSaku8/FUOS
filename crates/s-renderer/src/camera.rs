use nalgebra::Point3;

pub struct Camera {
    pub eye: Point3<f32>,

    pub target: Point3<f32>,

    pub up: nalgebra::Vector3<f32>,
}
