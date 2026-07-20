use ndarray::Array4;

pub struct BoundaryFile {
    pub u: Array4<f32>,

    pub v: Array4<f32>,

    pub w: Array4<f32>,

    pub temperature: Array4<f32>,

    pub salinity: Array4<f32>,
}
