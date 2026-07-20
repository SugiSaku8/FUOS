use ndarray::{Array1, Array2};

pub struct GridFile {
    pub x: Array1<f64>,

    pub y: Array1<f64>,

    pub z: Array1<f64>,

    pub depth: Array2<f32>,

    pub mask: Array2<u8>,

    pub dx: Array2<f32>,

    pub dy: Array2<f32>,

    pub dz: Array1<f32>,
}
