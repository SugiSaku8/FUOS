use crate::{Grid4D, VelocityField};

/// 駿河湾へ流入する黒潮の簡易モデルを生成する。
///
/// 南側境界から北東方向へ流入する。
pub fn create_kuroshio(grid: Grid4D) -> VelocityField {
    let mut velocity = VelocityField::new(grid);

    let shape = velocity.shape();

    let nx = shape[0];
    let ny = shape[1];
    let nz = shape[2];

    //
    // 値の書き込み
    //
    {
        let u = velocity.u_mut().time_slice_mut(0).unwrap();

        for z in 0..nz {
            //
            // 表層ほど強い
            //
            let depth_factor = 1.0 - z as f32 / nz as f32;

            for y in 0..ny {
                for x in 0..nx {
                    let index = z * nx * ny + y * nx + x;

                    //
                    // 南側入口
                    //
                    let south = y as f32 / ny as f32;

                    u[index] = 0.8 * depth_factor * (1.0 - south);
                }
            }
        }
    }

    {
        let v = velocity.v_mut().time_slice_mut(0).unwrap();

        for z in 0..nz {
            let depth_factor = 1.0 - z as f32 / nz as f32;

            for y in 0..ny {
                for x in 0..nx {
                    let index = z * nx * ny + y * nx + x;

                    //
                    // 少し北向き
                    //
                    v[index] = 0.3 * depth_factor;
                }
            }
        }
    }

    {
        let w = velocity.w_mut().time_slice_mut(0).unwrap();

        for value in w.iter_mut() {
            *value = 0.0;
        }
    }

    velocity
}
