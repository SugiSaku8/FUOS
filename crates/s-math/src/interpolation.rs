/// Linear interpolation
///
/// a ---- b
///   t=0.0 -> a
///   t=1.0 -> b
///
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Trilinear interpolation
///
/// 3D cube:
///
/// c000 ---- c100
///  |          |
/// c010 ---- c110
///
/// c001 ---- c101
///  |          |
/// c011 ---- c111
///
/// tx,ty,tz : 0.0 ~ 1.0
///
pub fn trilinear(values: [f32; 8], tx: f32, ty: f32, tz: f32) -> f32 {
    let c000 = values[0];
    let c100 = values[1];
    let c010 = values[2];
    let c110 = values[3];

    let c001 = values[4];
    let c101 = values[5];
    let c011 = values[6];
    let c111 = values[7];

    let x00 = lerp(c000, c100, tx);

    let x10 = lerp(c010, c110, tx);

    let x01 = lerp(c001, c101, tx);

    let x11 = lerp(c011, c111, tx);

    let y0 = lerp(x00, x10, ty);

    let y1 = lerp(x01, x11, ty);

    lerp(y0, y1, tz)
}
