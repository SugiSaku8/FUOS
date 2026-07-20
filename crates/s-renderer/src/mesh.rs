use crate::vertex::Vertex;

pub struct Mesh {
    pub vertices: Vec<Vertex>,

    pub indices: Vec<u32>,
}
