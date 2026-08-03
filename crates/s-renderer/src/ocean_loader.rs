use anyhow::Result;

use s_field::VelocityField;

use s_sof::package_reader::SofPackageReader;

pub struct OceanLoader;

impl OceanLoader {
    pub fn load_velocity(path: &str) -> Result<VelocityField> {
        let mut package = SofPackageReader::open(path)?;

        let velocity = package.read_boundary()?;

        Ok(velocity.velocity())
    }
}
