use s_field::{
    BoundaryField, Coordinates, Field4D, Grid4D, OceanField, SalinityField, TemperatureField,
    VelocityField,
};

use s_io::SioReader;

use crate::error::Result;

pub struct BoundaryReader {
    reader: SioReader,
}

impl BoundaryReader {
    pub fn open(path: &str) -> Result<Self> {
        let reader = SioReader::open(path)?;

        Ok(Self { reader })
    }

    pub fn read(&mut self) -> Result<OceanField> {
        //
        // Metadata
        //

        let sio_metadata = self.reader.metadata().clone();

        let mut metadata = s_field::Metadata::new();

        metadata.set_dataset_name(sio_metadata.name);

        metadata.set_title("Suruga Ocean Format Boundary Dataset");

        metadata.set_comment(sio_metadata.description);

        metadata.insert_attribute("sof_version", sio_metadata.version);

        //
        // Grid
        //

        let size = self.reader.read_dataset("grid_size")?;

        let spacing = self.reader.read_dataset("grid_spacing")?;

        let grid = Grid4D::new(
            [
                size[0] as usize,
                size[1] as usize,
                size[2] as usize,
                size[3] as usize,
            ],
            [spacing[0], spacing[1], spacing[2], spacing[3]],
        );

        //
        // Coordinates
        //

        let latitude = self
            .reader
            .read_dataset("latitude")?
            .into_iter()
            .map(|v| v as f64)
            .collect();

        let longitude = self
            .reader
            .read_dataset("longitude")?
            .into_iter()
            .map(|v| v as f64)
            .collect();

        let depth = self.reader.read_dataset("depth")?;

        let time = self
            .reader
            .read_dataset("time")?
            .into_iter()
            .map(|v| v as f64)
            .collect();

        let coordinates = Coordinates::new(grid, latitude, longitude, depth, time);

        //
        // Velocity
        //

        let u = self.reader.read_dataset("velocity_u")?;

        let v = self.reader.read_dataset("velocity_v")?;

        let w = self.reader.read_dataset("velocity_w")?;

        let mut velocity = VelocityField::new(*coordinates.grid());

        velocity.u_mut().replace(u);

        velocity.v_mut().replace(v);

        velocity.w_mut().replace(w);

        velocity.set_coordinates(coordinates.clone());
        //
        // Temperature
        //

        let temperature = self.reader.read_dataset("temperature")?;

        let temperature =
            TemperatureField::from_field(Field4D::from_vec(*coordinates.grid(), temperature));

        //
        // Salinity
        //

        let salinity = self.reader.read_dataset("salinity")?;

        let salinity = SalinityField::from_field(Field4D::from_vec(*coordinates.grid(), salinity));
        let mut boundary = BoundaryField::zeros(*coordinates.grid());
        // Older SOF boundary files did not store edge arrays; retain zero-boundary
        // compatibility for those files and read explicit faces when available.
        let has_boundary_faces = self.reader.datasets().iter().any(|d| d.name == "boundary_west_u");
        if has_boundary_faces {
            boundary.west.u_mut().replace(self.reader.read_dataset("boundary_west_u")?);
            boundary.west.v_mut().replace(self.reader.read_dataset("boundary_west_v")?);
            boundary.west.w_mut().replace(self.reader.read_dataset("boundary_west_w")?);
            boundary.east.u_mut().replace(self.reader.read_dataset("boundary_east_u")?);
            boundary.east.v_mut().replace(self.reader.read_dataset("boundary_east_v")?);
            boundary.east.w_mut().replace(self.reader.read_dataset("boundary_east_w")?);
            boundary.north.u_mut().replace(self.reader.read_dataset("boundary_north_u")?);
            boundary.north.v_mut().replace(self.reader.read_dataset("boundary_north_v")?);
            boundary.north.w_mut().replace(self.reader.read_dataset("boundary_north_w")?);
            boundary.south.u_mut().replace(self.reader.read_dataset("boundary_south_u")?);
            boundary.south.v_mut().replace(self.reader.read_dataset("boundary_south_v")?);
            boundary.south.w_mut().replace(self.reader.read_dataset("boundary_south_w")?);
        }
        //
        // OceanField
        //

        Ok(OceanField::new(
            coordinates,
            metadata,
            velocity,
            temperature,
            salinity,
            boundary,
        ))
    }
}
