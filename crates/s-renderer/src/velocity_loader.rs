use anyhow::Result;

use s_field::{Grid4D, VelocityField};

use s_io::SioReader;

pub fn load_velocity(path: &str, grid: Grid4D) -> Result<VelocityField> {
    let mut reader = SioReader::open(path)?;

    let u = reader.read_array("velocity_u")?;

    let v = reader.read_array("velocity_v")?;

    let w = reader.read_array("velocity_w")?;

    let mut velocity = VelocityField::new(grid);

    velocity.u_mut().as_mut_slice().copy_from_slice(&u);

    velocity.v_mut().as_mut_slice().copy_from_slice(&v);

    velocity.w_mut().as_mut_slice().copy_from_slice(&w);

    Ok(velocity)
}
