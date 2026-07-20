use ndarray::{Array1, Array2};

pub struct OceanGrid {
    /// X座標(m)
    pub x: Array1<f64>,

    /// Y座標(m)
    pub y: Array1<f64>,

    /// Z座標(m)
    pub z: Array1<f64>,

    /// 海底深度(m)
    pub depth: Array2<f32>,

    /// 海陸マスク
    pub mask: Array2<u8>,

    /// セル幅
    pub dx: Array2<f32>,

    pub dy: Array2<f32>,

    /// 鉛直セル厚
    pub dz: Array1<f32>,
}
impl OceanGrid {
    pub fn width(&self) -> usize {
        self.depth.ncols()
    }

    pub fn height(&self) -> usize {
        self.depth.nrows()
    }

    pub fn layers(&self) -> usize {
        self.z.len()
    }
}
impl OceanGrid {
    #[inline]
    pub fn depth(&self, x: usize, y: usize) -> f32 {
        self.depth[[y, x]]
    }
}
impl OceanGrid {
    #[inline]
    pub fn is_sea(&self, x: usize, y: usize) -> bool {
        self.mask[[y, x]] == 1
    }
}
impl OceanGrid {
    pub fn dx(&self, x: usize, y: usize) -> f32 {
        self.dx[[y, x]]
    }

    pub fn dy(&self, x: usize, y: usize) -> f32 {
        self.dy[[y, x]]
    }

    pub fn dz(&self, z: usize) -> f32 {
        self.dz[z]
    }
}
impl OceanGrid {
    pub fn position(&self, x: usize, y: usize) -> [f32; 3] {
        [self.x[x] as f32, self.y[y] as f32, -self.depth(x, y)]
    }
}
impl OceanGrid {
    pub fn in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && x < self.width() as isize && y < self.height() as isize
    }
}
