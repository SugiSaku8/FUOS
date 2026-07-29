use s_field::Field4D;

pub fn field4d_time_slice(field: &Field4D<f32>, time: usize) -> Option<Vec<f32>> {
    Some(field.time_slice(time)?.to_vec())
}
