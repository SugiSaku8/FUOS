use s_math::Vector3;

#[test]
fn test_add() {
    let a = Vector3::new(1.0, 2.0, 3.0);

    let b = Vector3::new(2.0, 3.0, 4.0);

    let c = a.add(b);

    assert_eq!(c, Vector3::new(3.0, 5.0, 7.0));
}

#[test]
fn test_length() {
    let v = Vector3::new(3.0, 4.0, 0.0);

    assert_eq!(v.length(), 5.0);
}

#[test]
fn test_normalize() {
    let v = Vector3::new(0.0, 3.0, 0.0);

    let n = v.normalize();

    assert_eq!(n, Vector3::new(0.0, 1.0, 0.0));
}
