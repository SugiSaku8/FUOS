use ndarray::Array2;

#[derive(Clone)]
pub struct OceanMask {
    pub sea: Array2<bool>,
}

impl OceanMask {
    pub fn from_bathymetry(depth: &Array2<f32>) -> Self {
        let sea = depth.mapv(|v| v < 0.0);

        Self { sea }
    }
}
