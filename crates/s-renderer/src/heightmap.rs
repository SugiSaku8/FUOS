#[derive(Clone)]
pub struct HeightMap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

impl HeightMap {
    pub fn new(width: usize, height: usize, data: Vec<f32>) -> Self {
        assert_eq!(data.len(), width * height);

        Self {
            width,
            height,
            data,
        }
    }

    //
    // テスト用地形
    //
    pub fn test() -> Self {
        let width = 128;
        let height = 128;

        let mut data = Vec::new();

        for z in 0..height {
            for x in 0..width {
                let xf = x as f32 / width as f32;

                let zf = z as f32 / height as f32;

                let h = (xf * 12.0).sin() * (zf * 12.0).cos() * 20.0;

                data.push(h);
            }
        }

        Self {
            width,
            height,
            data,
        }
    }

    pub fn get(&self, x: usize, z: usize) -> f32 {
        self.data[z * self.width + x]
    }

    pub fn min_max(&self) -> (f32, f32) {
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for v in &self.data {
            min = min.min(*v);
            max = max.max(*v);
        }

        (min, max)
    }
}
