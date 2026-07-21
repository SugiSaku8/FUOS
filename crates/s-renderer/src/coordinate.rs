//
// coordinate.rs
//

//
// 緯度経度範囲
//
#[derive(Clone, Copy, Debug)]
pub struct GeoBounds {
    pub lon_min: f64,
    pub lon_max: f64,

    pub lat_min: f64,
    pub lat_max: f64,
}

impl GeoBounds {

    pub fn new(
        lon_min: f64,
        lon_max: f64,
        lat_min: f64,
        lat_max: f64,
    ) -> Self {

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



//
// lon/lat → 3D座標
//
pub struct GeoConverter {

    bounds: GeoBounds,

    //
    // 水平方向スケール
    //
    horizontal_scale: f32,


    //
    // 高度倍率
    //
    vertical_scale: f32,
}



impl GeoConverter {


    pub fn new(bounds: GeoBounds) -> Self {

        Self {

            bounds,


            //
            // 111km/degree
            //
            // 表示用に縮小
            //
            horizontal_scale: 1.0,


            //
            // 海底地形を見えるように10倍
            //
            vertical_scale: 1.0,

        }

    }



    //
    // lon lat height
    //
    // ↓
    //
    // [x,y,z]
    //
    pub fn convert(
        &self,
        lon: f64,
        lat: f64,
        height: f32,
    ) -> [f32;3] {


        let center_lon =
            self.bounds.center_lon();


        let center_lat =
            self.bounds.center_lat();



        //
        // 経度方向
        //
        // 地球曲率補正
        //
        let x = -(
            (lon - center_lon)
                * self.horizontal_scale as f64
                * 111_000.0
                * center_lat.to_radians().cos()
        ) as f32;

        //
        // 緯度方向
        //
        let z =
            (
                (lat-center_lat)
                *
                111_000.0
            ) as f32
            *
            self.horizontal_scale;



        //
        // 高度
        //
        // GEBCO:
        //
        // 海底 → 負
        // 陸地 → 正
        //
        let y =
            height
            *
            self.vertical_scale;



        [
            x,
            y,
            z,
        ]

    }



    pub fn bounds(&self)->GeoBounds {

        self.bounds

    }



}
