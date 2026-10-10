use anyhow::Result;
use ndarray::Array2;
use netcdf::open;


pub struct GebcoLoader;


impl GebcoLoader {

    pub fn load(
        path:&str
    )->Result<Vec<f32>> {


        let file =
            open(path)?;


        let var =
            file.variable("elevation")
            .unwrap();


        let data =
            var.get_values::<f32, _>(..)?;


        Ok(data)

    }

}
