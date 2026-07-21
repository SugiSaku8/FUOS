use crate::field4d::Field4D;

#[derive(Clone, Debug)]
pub struct TemperatureField {
    values: Field4D<f32>,
}

impl TemperatureField {
    pub fn new(values: Field4D<f32>) -> Self {
        Self { values }
    }

    #[inline]
    pub fn get(&self, t: usize, z: usize, y: usize, x: usize) -> Option<f32> {
        self.values.get(t, z, y, x).copied()
    }

    #[inline]
    pub fn values(&self) -> &Field4D<f32> {
        &self.values
    }
}
