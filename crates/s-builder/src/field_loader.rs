use anyhow::Result;

use s_field::{
    Grid4D,
    ScalarField,
};


pub struct FieldLoader;


impl FieldLoader {

    /// Loads raw f32 data into ScalarField.
    pub fn load(
        grid: Grid4D,
        data: Vec<f32>,
    ) -> Result<ScalarField<f32>> {

        let expected =
            grid.width()
            * grid.height()
            * grid.depth()
            * grid.time();


        if data.len() != expected {
            anyhow::bail!(
                "FieldLoader: size mismatch (expected {}, got {})",
                expected,
                data.len()
            );
        }


        let mut field = ScalarField::zeros(grid);


        field.replace(data);


        Ok(field)
    }


    pub fn scalar(
        grid: Grid4D,
        data: Vec<f32>,
    ) -> Result<ScalarField<f32>> {

        Self::load(grid, data)
    }


    pub fn velocity_component(
        grid: Grid4D,
        data: Vec<f32>,
    ) -> Result<ScalarField<f32>> {

        Self::load(grid, data)
    }
}
