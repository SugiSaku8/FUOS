use s_math::Matrix3;

#[test]
fn test_identity() {
    let m = Matrix3::identity();

    assert_eq!(m.data[0][0], 1.0);

    assert_eq!(m.data[1][1], 1.0);

    assert_eq!(m.data[2][2], 1.0);
}

#[test]
fn test_transpose() {
    let m = Matrix3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);

    let t = m.transpose();

    assert_eq!(t.data[0][1], 4.0);
}
