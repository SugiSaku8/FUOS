use ndarray::Array3;

pub struct BoundaryCondition {
    pub u: Array3<f32>,

    pub v: Array3<f32>,

    pub w: Array3<f32>,

    pub temperature: Array3<f32>,

    pub salinity: Array3<f32>,
}
