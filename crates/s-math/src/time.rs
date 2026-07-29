use crate::vector::Vector3;

pub fn euler_step(current: f32, derivative: f32, dt: f32) -> f32 {
    current + derivative * dt
}

pub fn euler_vector3(current: Vector3, derivative: Vector3, dt: f32) -> Vector3 {
    current.add(derivative.scale(dt))
}
