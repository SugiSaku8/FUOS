use crate::reader::Reader;
use S_grid::bathymetry::Bathymetry;

pub struct GebcoReader;

impl Reader<Bathymetry> for GebcoReader {
    fn read(path: &str) -> anyhow::Result<Bathymetry> {
        let file = netcdf::open(path)?;

        todo!("GEBCO読み込み")
    }
}
