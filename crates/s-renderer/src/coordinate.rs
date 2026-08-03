//! Geographic coordinate conversion for rendering.

/// Geographic bounds.
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
}

/// Converts geographic coordinates into renderer coordinates.
///
/// Coordinate system:
///
/// x : east
/// y : height
/// z : south
///
#[derive(Clone, Debug)]
pub struct GeoConverter {
    bounds: GeoBounds,

    scale: f32,

    sea_level: f32,
}

impl GeoConverter {
    pub fn new(bounds: GeoBounds, scale: f32) -> Self {
        Self {
            bounds,

            scale,

            sea_level: 0.0,
        }
    }

    /// Converts longitude and latitude to world coordinates.
    pub fn geo_to_world(&self, longitude: f64, latitude: f64) -> [f32; 3] {
        let x = ((longitude - self.bounds.lon_min) / (self.bounds.lon_max - self.bounds.lon_min))
            as f32
            * self.scale;

        let z = ((self.bounds.lat_max - latitude) / (self.bounds.lat_max - self.bounds.lat_min))
            as f32
            * self.scale;

        [x, self.sea_level, z]
    }

    /// Converts ocean point.
    ///
    /// depth:
    /// positive downward
    ///
    pub fn ocean_to_world(&self, longitude: f64, latitude: f64, depth: f32) -> [f32; 3] {
        let mut p = self.geo_to_world(longitude, latitude);

        //
        // underwater
        //
        p[1] = -depth;

        p
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn bounds(&self) -> GeoBounds {
        self.bounds
    }
}
