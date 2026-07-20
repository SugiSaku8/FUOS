pub struct Geometry<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
}

impl<V> Geometry<V> {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}
