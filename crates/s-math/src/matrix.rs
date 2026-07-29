#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3 {
    pub data: [[f32; 3]; 3],
}

impl Matrix3 {
    pub fn new(data: [[f32; 3]; 3]) -> Self {
        Self { data }
    }

    pub fn identity() -> Self {
        Self {
            data: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn multiply(self, other: Self) -> Self {
        let mut result = [[0.0; 3]; 3];

        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }

        Self { data: result }
    }

    pub fn transpose(self) -> Self {
        let mut result = [[0.0; 3]; 3];

        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.data[j][i];
            }
        }

        Self { data: result }
    }
}
