//
// coordinate.rs
//

#[derive(Clone, Copy, Debug)]
pub struct GeoBounds {
    pub lon_min: f64,
    pub lon_max: f64,

    pub lat_min: f64,
    pub lat_max: f64,
}

impl GeoBounds {
    pub fn new(lon_min: f64, lon_max: f64, lat_min: f64, lat_max: f64) -> Self {
        Self {
            lon_min,
            lon_max,
            lat_min,
            lat_max,
        }
    }

    pub fn center_lon(&self) -> f64 {
        (self.lon_min + self.lon_max) * 0.5
    }

    pub fn center_lat(&self) -> f64 {
        (self.lat_min + self.lat_max) * 0.5
    }
}

pub struct GeoConverter {
    bounds: GeoBounds,

    /// 水平方向倍率
    horizontal_scale: f32,

    /// 高さ倍率
    vertical_scale: f32,

    /// 緯度1度あたりの距離(m)
    meters_per_deg_lat: f64,

    /// 経度1度あたりの距離(m)
    meters_per_deg_lon: f64,
}

impl GeoConverter {
    pub fn new(bounds: GeoBounds) -> Self {
        let lat = bounds.center_lat().to_radians();

        // WGS84近似式
        let meters_per_deg_lat = 111132.92 - 559.82 * (2.0 * lat).cos() + 1.175 * (4.0 * lat).cos()
            - 0.0023 * (6.0 * lat).cos();

        let meters_per_deg_lon =
            111412.84 * lat.cos() - 93.5 * (3.0 * lat).cos() + 0.118 * (5.0 * lat).cos();

        Self {
            bounds,

            horizontal_scale: 1.0,

            // 地形を見やすくするため
            vertical_scale: 3.0,

            meters_per_deg_lat,
            meters_per_deg_lon,
        }
    }

    pub fn convert(&self, lon: f64, lat: f64, height: f32) -> [f32; 3] {
        let center_lon = self.bounds.center_lon();
        let center_lat = self.bounds.center_lat();

        // 東西（東が+X）
        let x =
            -((lon - center_lon) * self.meters_per_deg_lon * self.horizontal_scale as f64) as f32;

        // 南北（北が+Z）
        let z =
            ((lat - center_lat) * self.meters_per_deg_lat * self.horizontal_scale as f64) as f32;

        // 高度
        let y = height * self.vertical_scale;

        [x, y, z]
    }

    pub fn bounds(&self) -> GeoBounds {
        self.bounds
    }
}
