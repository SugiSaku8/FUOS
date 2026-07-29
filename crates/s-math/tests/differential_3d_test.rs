use s_math::differential::derivative_x;

#[test]
fn test_derivative_x() {
    let nx = 4;
    let ny = 1;
    let nz = 1;

    // f(x)=2x

    let field = vec![0.0, 2.0, 4.0, 6.0];

    let result = derivative_x(&field, nx, ny, nz, 1.0);

    for v in result {
        assert!((v - 2.0).abs() < 0.001);
    }
}

