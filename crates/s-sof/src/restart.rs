use ndarray::Array4;

pub struct RestartFile {
    pub velocity_u: Array4<f32>,

    pub velocity_v: Array4<f32>,

    pub velocity_w: Array4<f32>,

    pub temperature: Array4<f32>,

    pub salinity: Array4<f32>,

    pub simulation_time: f64,
}
