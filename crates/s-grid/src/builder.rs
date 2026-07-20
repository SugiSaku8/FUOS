use crate::bathymetry::Bathymetry;
use crate::geometry::CellGeometry;
use crate::grid::OceanGrid;
use crate::mask::OceanMask;

pub struct GridBuilder {
    bathymetry: Bathymetry,

    dz: f32,
}

impl GridBuilder {
    pub fn new(bathymetry: Bathymetry) -> Self {
        Self {
            bathymetry,

            dz: 20.0,
        }
    }

    pub fn vertical_resolution(mut self, dz: f32) -> Self {
        self.dz = dz;

        self
    }

    pub fn build(self) -> OceanGrid {
        let nx = self.bathymetry.width();

        let ny = self.bathymetry.height();

        let nz = 300;

        OceanGrid {
            mask: OceanMask::from_bathymetry(&self.bathymetry.elevation),

            geometry: CellGeometry {
                dx: 500.0,

                dy: 500.0,

                dz: self.dz,
            },

            bathymetry: self.bathymetry,

            nx,

            ny,

            nz,
        }
    }
}
