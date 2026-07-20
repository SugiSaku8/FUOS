use ndarray::Array2;

/// 海底標高
///
/// 値は m
/// 海は負
/// 陸は正
#[derive(Clone)]
pub struct Bathymetry {
    pub elevation: Array2<f32>,
}

impl Bathymetry {
    pub fn width(&self) -> usize {
        self.elevation.shape()[0]
    }

    pub fn height(&self) -> usize {
        self.elevation.shape()[1]
    }
}
