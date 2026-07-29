use s_math::Vector3;

use s_math::time::{euler_step, euler_vector3};

#[test]
fn test_euler() {
    let result = euler_step(10.0, 2.0, 0.5);

    assert_eq!(result, 11.0);
}

#[test]
fn test_euler_vector() {
    let velocity = Vector3::new(1.0, 0.0, 0.0);

    let acceleration = Vector3::new(2.0, 0.0, 0.0);

    let result = euler_vector3(velocity, acceleration, 0.5);

    assert_eq!(result.x, 2.0);
}
