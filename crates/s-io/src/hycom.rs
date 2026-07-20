use crate::boundary::BoundaryCondition;
use crate::reader::Reader;

pub struct HycomReader;

impl Reader<BoundaryCondition> for HycomReader {
    fn read(path: &str) -> anyhow::Result<BoundaryCondition> {
        let file = netcdf::open(path)?;

        todo!("HYCOM読み込み")
    }
}
