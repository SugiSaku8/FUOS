#[derive(Clone, Debug)]
pub struct Coordinates {
    pub latitude: Vec<f64>,

    pub longitude: Vec<f64>,

    pub depth: Vec<f32>,

    pub time: Vec<f64>,
}

impl Coordinates {
    pub fn new(latitude: Vec<f64>, longitude: Vec<f64>, depth: Vec<f32>, time: Vec<f64>) -> Self {
        Self {
            latitude,
            longitude,
            depth,
            time,
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.longitude.len()
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.latitude.len()
    }

    #[inline]
    pub fn depth_levels(&self) -> usize {
        self.depth.len()
    }

    #[inline]
    pub fn time_steps(&self) -> usize {
        self.time.len()
    }
}
