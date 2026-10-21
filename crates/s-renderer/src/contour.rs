use crate::coordinate::{GeoBounds, GeoConverter};
use crate::flow::FlowVertex;
use crate::heightmap::HeightMap;

/// 海底地形データから等深線を生成する。
///
/// # Arguments
///
/// * `heightmap` - 海底地形の標高データ（海底は負の値）
/// * `bounds` - 地理座標の範囲
/// * `vertical_scale` - 地形表示時の垂直方向の倍率
/// * `interval_m` - 等深線の間隔（メートル）
///
/// # Returns
///
/// 等深線を構成する線分の頂点配列。
pub fn generate_bathymetry_contours(
    heightmap: &HeightMap,
    bounds: GeoBounds,
    vertical_scale: f32,
    interval_m: f32,
) -> Vec<FlowVertex> {
    generate_bathymetry_contours_with_scale(heightmap, bounds, vertical_scale, interval_m, 100_000.0)
}

/// 指定ワールドスケールで海底等深線を生成する。
pub fn generate_bathymetry_contours_with_scale(
    heightmap: &HeightMap,
    bounds: GeoBounds,
    vertical_scale: f32,
    interval_m: f32,
    world_scale: f32,
) -> Vec<FlowVertex> {
    let width = heightmap.width;
    let height = heightmap.height;

    if width < 2 || height < 2 || interval_m <= 0.0 {
        return Vec::new();
    }

    let converter = GeoConverter::new(bounds, world_scale);
    let mut vertices = Vec::new();

    // 250 mなど、指定された間隔ごとに等深線を生成する。
    for z in 0..height - 1 {
        let lat0 =
            bounds.lat_min + (bounds.lat_max - bounds.lat_min) * z as f64 / (height - 1) as f64;

        let lat1 = bounds.lat_min
            + (bounds.lat_max - bounds.lat_min) * (z + 1) as f64 / (height - 1) as f64;

        for x in 0..width - 1 {
            let lon0 =
                bounds.lon_min + (bounds.lon_max - bounds.lon_min) * x as f64 / (width - 1) as f64;

            let lon1 = bounds.lon_min
                + (bounds.lon_max - bounds.lon_min) * (x + 1) as f64 / (width - 1) as f64;

            // セルの四隅。
            let corners = [
                (lon0, lat0, heightmap.get(x, z)),
                (lon1, lat0, heightmap.get(x + 1, z)),
                (lon1, lat1, heightmap.get(x + 1, z + 1)),
                (lon0, lat1, heightmap.get(x, z + 1)),
            ];

            // 標高データが不正なセルはスキップする。
            if corners.iter().any(|(_, _, h)| !h.is_finite()) {
                continue;
            }

            let min_height = corners
                .iter()
                .map(|(_, _, h)| *h)
                .fold(f32::INFINITY, f32::min);

            let max_height = corners
                .iter()
                .map(|(_, _, h)| *h)
                .fold(f32::NEG_INFINITY, f32::max);

            // 海底を含まないセルには等深線を生成しない。
            if min_height >= 0.0 {
                continue;
            }

            // 対角線で四角形を2つの三角形に分割する。
            let triangles = [[0_usize, 1_usize, 2_usize], [0_usize, 2_usize, 3_usize]];

            let first_level = (min_height / interval_m).ceil() as i32;
            let last_level = (max_height / interval_m).floor() as i32;

            for level_index in first_level..=last_level {
                let target = level_index as f32 * interval_m;

                // 海底の等深線だけを描画する。
                if target >= 0.0 {
                    continue;
                }

                for triangle in triangles {
                    let mut intersections: Vec<(f64, f64)> = Vec::with_capacity(3);

                    for [a_idx, b_idx] in [[0_usize, 1_usize], [1, 2], [2, 0]] {
                        let a = corners[triangle[a_idx]];
                        let b = corners[triangle[b_idx]];

                        let h_a = a.2;
                        let h_b = b.2;

                        // 等深線と交差する辺を判定する。
                        let crosses =
                            (h_a < target && h_b >= target) || (h_b < target && h_a >= target);

                        if !crosses {
                            continue;
                        }

                        let denominator = h_b - h_a;

                        if denominator.abs() < f32::EPSILON {
                            continue;
                        }

                        let t = ((target - h_a) / denominator) as f64;

                        let lon = a.0 + (b.0 - a.0) * t;
                        let lat = a.1 + (b.1 - a.1) * t;

                        intersections.push((lon, lat));
                    }

                    // 三角形と等深線の交点が2つなら線分を作る。
                    if intersections.len() != 2 {
                        continue;
                    }

                    let (lon_a, lat_a) = intersections[0];
                    let (lon_b, lat_b) = intersections[1];

                    let mut world_a = converter.geo_to_world(lon_a, lat_a);
                    let mut world_b = converter.geo_to_world(lon_b, lat_b);

                    // 地形表面との重なりを避けるため、わずかに下げる。
                    let y = target * vertical_scale + 0.5;
                    world_a[1] = y;
                    world_b[1] = y;

                    vertices.push(FlowVertex { position: world_a, color: FlowVertex::CONTOUR });

                    vertices.push(FlowVertex { position: world_b, color: FlowVertex::CONTOUR });
                }
            }
        }
    }

    vertices
}
