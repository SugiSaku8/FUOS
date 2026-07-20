use std::f32::consts::PI;

//
// 緯度経度範囲
//
#[derive(Clone, Copy)]
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

    //
    // 中心経度
    //
    pub fn center_lon(&self) -> f64 {
        (self.lon_min + self.lon_max) * 0.5
    }

    //
    // 中心緯度
    //
    pub fn center_lat(&self) -> f64 {
        (self.lat_min + self.lat_max) * 0.5
    }
}

//
// lon/lat → ローカル3D
//
pub struct GeoConverter {
    bounds: GeoBounds,

    //
    // 1度あたりの距離
    //
    scale: f32,
}

impl GeoConverter {
    pub fn new(bounds: GeoBounds) -> Self {
        //
        // 1度 ≒ 111km
        //
        let scale = 111_000.0;

        Self { bounds, scale }
    }

    //
    // 経度緯度高度を3Dへ
    //
    pub fn convert(&self, lon: f64, lat: f64, height: f32) -> [f32; 3] {
        let center_lon = self.bounds.center_lon();

        let center_lat = self.bounds.center_lat();

        //
        // 経度方向
        //
        // 緯度による補正
        //
        let x = ((lon - center_lon) * self.scale as f64 * center_lat.to_radians().cos()) as f32;

        //
        // 標高
        //
        let y = height;

        //
        // 緯度方向
        //
        let z = ((lat - center_lat) * self.scale as f64) as f32;

        [x, y, z]
    }

    //
    // 座標範囲を取得
    //
    pub fn bounds(&self) -> GeoBounds {
        self.bounds
    }
}
