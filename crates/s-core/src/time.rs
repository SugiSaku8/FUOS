#[derive(Clone)]
pub struct TimeStep {
    /// 秒
    pub dt: f32,

    /// 現在時刻
    pub time: f64,
}

impl TimeStep {
    pub fn advance(&mut self) {
        self.time += self.dt as f64;
    }
}
