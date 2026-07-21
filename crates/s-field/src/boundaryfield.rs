pub struct BoundaryField {
    pub latitude: Vec<f64>,
    pub longitude: Vec<f64>,
    pub depth: Vec<f32>,
    pub time: Vec<f64>,

    pub velocity: VelocityField,
    pub temperature: TemperatureField,
    pub salinity: SalinityField,
}
