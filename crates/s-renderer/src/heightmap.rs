#[derive(Clone, Debug)]
pub struct HeightMap {
    pub width: usize,

    pub height: usize,

    //
    // row-major
    //
    // index = z * width + x
    //
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
    // 値取得
    //
    pub fn get(&self, x: usize, z: usize) -> f32 {
        self.data[z * self.width + x]
    }

    //
    // 値設定
    //
    pub fn set(&mut self, x: usize, z: usize, value: f32) {
        self.data[z * self.width + x] = value;
    }

    //
    // テスト地形
    //
    pub fn test() -> Self {
        let width = 128;

        let height = 128;

        let mut data = Vec::with_capacity(width * height);

        for z in 0..height {
            for x in 0..width {
                let xf = x as f32 / width as f32;

                let zf = z as f32 / height as f32;

                //
                // 仮の海底地形
                //
                let h = ((xf * 12.0).sin() * (zf * 12.0).cos() * 20.0);

                data.push(h);
            }
        }

        Self::new(width, height, data)
    }

    //
    // 最小値・最大値
    //
    pub fn min_max(&self) -> (f32, f32) {
        let mut min = f32::MAX;

        let mut max = f32::MIN;

        for value in &self.data {
            min = min.min(*value);

            max = max.max(*value);
        }

        (min, max)
    }

    //
    // 正規化
    //
    // 0.0〜1.0へ変換
    //
    pub fn normalized(&self) -> Self {
        let (min, max) = self.min_max();

        let range = max - min;

        let data = self.data.iter().map(|v| (v - min) / range).collect();

        Self {
            width: self.width,

            height: self.height,

            data,
        }
    }
}
