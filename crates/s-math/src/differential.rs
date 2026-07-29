pub fn derivative_x(field: &[f32], nx: usize, ny: usize, nz: usize, dx: f32) -> Vec<f32> {
    let mut result = vec![0.0; field.len()];

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let i = z * nx * ny + y * nx + x;

                result[i] = if x == 0 {
                    (field[i + 1] - field[i]) / dx
                } else if x == nx - 1 {
                    (field[i] - field[i - 1]) / dx
                } else {
                    (field[i + 1] - field[i - 1]) / (2.0 * dx)
                };
            }
        }
    }

    result
}

pub fn derivative_y(field: &[f32], nx: usize, ny: usize, nz: usize, dy: f32) -> Vec<f32> {
    let mut result = vec![0.0; field.len()];

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let i = z * nx * ny + y * nx + x;

                result[i] = if y == 0 {
                    (field[i + nx] - field[i]) / dy
                } else if y == ny - 1 {
                    (field[i] - field[i - nx]) / dy
                } else {
                    (field[i + nx] - field[i - nx]) / (2.0 * dy)
                };
            }
        }
    }

    result
}

pub fn derivative_z(field: &[f32], nx: usize, ny: usize, nz: usize, dz: f32) -> Vec<f32> {
    let mut result = vec![0.0; field.len()];

    let layer = nx * ny;

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let i = z * layer + y * nx + x;

                result[i] = if z == 0 {
                    (field[i + layer] - field[i]) / dz
                } else if z == nz - 1 {
                    (field[i] - field[i - layer]) / dz
                } else {
                    (field[i + layer] - field[i - layer]) / (2.0 * dz)
                };
            }
        }
    }

    result
}
