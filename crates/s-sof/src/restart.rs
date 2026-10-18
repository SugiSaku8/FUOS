use std::path::Path;

use s_field::OceanField;

use crate::error::Result;

use crate::writer::BoundaryWriter;
use crate::reader::BoundaryReader;



pub struct RestartWriter;


impl RestartWriter {


    pub fn write<P:AsRef<Path>>(
        path:P,
        field:&OceanField
    )->Result<()> {


        let mut writer =
            BoundaryWriter::create(path)?;


        writer.write(field)?;


        writer.finish()

    }

}



pub struct RestartReader;



impl RestartReader {


    pub fn read<P:AsRef<Path>>(
        path:P
    )->Result<OceanField>{


        let mut reader =
            BoundaryReader::open(
                path.as_ref()
                    .to_str()
                    .unwrap()
            )?;


        reader.read()

    }


}
