use crate::bathymetry::Bathymetry;
use crate::geometry::CellGeometry;
use crate::mask::OceanMask;

#[derive(Clone)]
pub struct OceanGrid {
    pub bathymetry: Bathymetry,

    pub mask: OceanMask,

    pub geometry: CellGeometry,

    pub nx: usize,

    pub ny: usize,

    pub nz: usize,
}
