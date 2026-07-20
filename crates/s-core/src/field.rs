use ndarray::Array3;

/// 任意の3次元フィールド
#[derive(Clone)]
pub struct Field3D<T> {
    pub data: Array3<T>,
}

impl<T: Clone> Field3D<T> {
    pub fn new(nx: usize, ny: usize, nz: usize, value: T) -> Self {
        Self {
            data: Array3::from_elem((nx, ny, nz), value),
        }
    }
}
