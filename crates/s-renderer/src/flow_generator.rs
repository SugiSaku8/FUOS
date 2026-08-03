use crate::{coordinate::GeoConverter, flow::FlowVertex};

use s_field::VelocityField;

/// CPU上でVelocityFieldから流線を生成する。
///
/// VelocityField:
///
/// longitude
/// latitude
/// depth
///
/// ↓
///
/// GeoConverter
///
/// ↓
///
/// Renderer world coordinates
///
pub struct FlowGenerator;

impl FlowGenerator {
    /// Generates flow line vertices.
    ///
    /// seeds:
    ///
    /// [
    /// longitude,
    /// latitude,
    /// depth
    /// ]
    ///
    pub fn generate(
        velocity: &VelocityField,

        converter: &GeoConverter,

        seeds: &[[f32; 3]],

        step: f32,

        length: usize,

        time: f64,
    ) -> Vec<FlowVertex> {
        let mut vertices = Vec::new();

        for seed in seeds {
            let mut p = *seed;

            for _ in 0..length {
                let vel = match velocity.sample(p[0] as f64, p[1] as f64, p[2], time) {
                    Some(v) => v,

                    None => {
                        break;
                    }
                };

                let next = [
                    p[0] + vel[0] * step,
                    p[1] + vel[1] * step,
                    p[2] + vel[2] * step,
                ];

                //
                // 地理座標 → GPU座標
                //

                let world_start = converter.ocean_to_world(p[0] as f64, p[1] as f64, p[2]);

                let world_end = converter.ocean_to_world(next[0] as f64, next[1] as f64, next[2]);

                vertices.push(FlowVertex {
                    position: world_start,
                });

                vertices.push(FlowVertex {
                    position: world_end,
                });

                p = next;
            }
        }

        vertices
    }
}
