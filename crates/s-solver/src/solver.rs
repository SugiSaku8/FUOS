use s_field::OceanField;

pub struct OceanSolver {
    dt: f32,
}

impl OceanSolver {
    pub fn new(dt: f32) -> Self {
        Self { dt }
    }

    pub fn step(&mut self, ocean: &mut OceanField) {

        /*
            ここで

            1.
            VelocityField取得


            2.
            GPU upload


            3.
            Gradient


            4.
            Advection


            5.
            Euler


            6.
            Field更新

        */
    }
}
