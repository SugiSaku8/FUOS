use crate::{coordinate::GeoConverter, flow::FlowVertex};

use s_field::VelocityField;

/// terrain.rs と同じ深度方向の表示倍率。
const OCEAN_VERTICAL_SCALE: f32 = 5.0;

/// 静的な流線を生成する。
pub struct FlowGenerator;

impl FlowGenerator {
    pub fn generate(
        velocity: &VelocityField,
        converter: &GeoConverter,
        seeds: &[[f32; 3]],
        dt_seconds: f32,
        length: usize,
        time: f64,
    ) -> Vec<FlowVertex> {
        let mut vertices = Vec::new();

        let Some(coordinates) = velocity.coordinates() else {
            return vertices;
        };

        let bounds = converter.bounds();

        for seed in seeds {
            let mut longitude = seed[0] as f64;
            let mut latitude = seed[1] as f64;
            let depth = seed[2];

            for _ in 0..length {
                if !coordinates.contains(longitude, latitude, depth) {
                    break;
                }

                if longitude < bounds.lon_min
                    || longitude > bounds.lon_max
                    || latitude < bounds.lat_min
                    || latitude > bounds.lat_max
                {
                    break;
                }

                let Some(vel) = velocity.sample(longitude, latitude, depth, time) else {
                    break;
                };

                let u = vel[0] as f64;
                let v = vel[1] as f64;

                if !u.is_finite() || !v.is_finite() || u.hypot(v) < 0.02 {
                    break;
                }

                let east_m = u * dt_seconds as f64;
                let north_m = v * dt_seconds as f64;

                let delta_lat = north_m / 110_540.0;

                let cos_lat = latitude.to_radians().cos();

                if cos_lat.abs() < 1.0e-6 {
                    break;
                }

                let delta_lon = east_m / (111_320.0 * cos_lat);

                let next_longitude = longitude + delta_lon;
                let next_latitude = latitude + delta_lat;

                if !coordinates.contains(next_longitude, next_latitude, depth) {
                    break;
                }

                if next_longitude < bounds.lon_min
                    || next_longitude > bounds.lon_max
                    || next_latitude < bounds.lat_min
                    || next_latitude > bounds.lat_max
                {
                    break;
                }

                let mut world_start = converter.ocean_to_world(longitude, latitude, depth);

                let mut world_end = converter.ocean_to_world(next_longitude, next_latitude, depth);

                // 海面をY=0、深度を下向きにし、海底地形と同じ垂直倍率で表示する。
                world_start[1] = -depth * OCEAN_VERTICAL_SCALE;
                world_end[1] = -depth * OCEAN_VERTICAL_SCALE;

                vertices.push(FlowVertex {
                    position: world_start,
                    color: FlowVertex::CYAN,
                });

                vertices.push(FlowVertex {
                    position: world_end,
                    color: FlowVertex::CYAN,
                });

                longitude = next_longitude;
                latitude = next_latitude;
            }
        }

        vertices
    }
}

/// 1つの流速粒子と、その軌跡を保持する。
struct FlowParticle {
    origin: [f64; 2],
    origin_depth: f32,

    longitude: f64,
    latitude: f64,
    depth: f32,

    trail: Vec<[f32; 3]>,
    flow_color: [f32; 3],
}

/// 流速データを使って粒子を時間発展させるシステム。
pub struct FlowParticleSystem {
    particles: Vec<FlowParticle>,
    max_trail_points: usize,
    vertical_scale: f32,
}

impl FlowParticleSystem {
    /// 初期位置から粒子を作成する。
    ///
    /// seedsは[経度、緯度、深度]。
    pub fn new(seeds: &[[f32; 3]], max_trail_points: usize) -> Self {
        Self::new_with_vertical_scale(seeds, max_trail_points, OCEAN_VERTICAL_SCALE)
    }

    /// 深度方向の表示倍率を指定して粒子を作成する。
    pub fn new_with_vertical_scale(seeds: &[[f32; 3]], max_trail_points: usize, vertical_scale: f32) -> Self {
        let particles = seeds
            .iter()
            .map(|seed| FlowParticle {
                origin: [seed[0] as f64, seed[1] as f64],
                origin_depth: seed[2],

                longitude: seed[0] as f64,
                latitude: seed[1] as f64,
                depth: seed[2],

                trail: Vec::new(),
                flow_color: FlowVertex::CYAN,
            })
            .collect();

        Self {
            particles,
            max_trail_points: max_trail_points.max(2),
            vertical_scale: if vertical_scale.is_finite() && vertical_scale > 0.0 { vertical_scale } else { OCEAN_VERTICAL_SCALE },
        }
    }

    /// 全粒子を1フレーム分だけ移動させる。
    pub fn update(
        &mut self,
        velocity: &VelocityField,
        converter: &GeoConverter,
        dt_seconds: f32,
        speed_multiplier: f32,
        time: f64,
    ) {
        self.update_with_temperature(velocity, &[], converter, dt_seconds, speed_multiplier, time);
    }

    /// Updates particles and colors them by temperature anomaly and vertical flow.
    /// Cyan/green highlights upward motion; warm/cold anomalies use orange/blue.
    pub fn update_with_temperature(
        &mut self,
        velocity: &VelocityField,
        temperature: &[f32],
        converter: &GeoConverter,
        dt_seconds: f32,
        speed_multiplier: f32,
        time: f64,
    ) {
        let Some(coordinates) = velocity.coordinates() else {
            return;
        };

        let bounds = converter.bounds();
        let temperature_reference = if temperature.is_empty() {
            10.0
        } else {
            let (sum, count) = temperature.iter().copied().filter(|value| value.is_finite())
                .fold((0.0_f32, 0_usize), |(sum, count), value| (sum + value, count + 1));
            if count == 0 { 10.0 } else { sum / count as f32 }
        };

        let dt = dt_seconds.clamp(0.0, 0.05) * speed_multiplier.max(0.0) * 100.0;

        if !dt.is_finite() || dt <= 0.0 {
            return;
        }

        for particle in &mut self.particles {
            let position_is_valid =
                coordinates.contains(particle.longitude, particle.latitude, particle.depth)
                    && particle.longitude >= bounds.lon_min
                    && particle.longitude <= bounds.lon_max
                    && particle.latitude >= bounds.lat_min
                    && particle.latitude <= bounds.lat_max;

            if !position_is_valid {
                Self::reset_particle(particle);
                continue;
            }

            let Some(vel) = velocity.sample_interpolated(
                particle.longitude,
                particle.latitude,
                particle.depth,
                time,
            ) else {
                Self::reset_particle(particle);
                continue;
            };

            let u = vel[0] as f64;
            let v = vel[1] as f64;
            let w = vel[2] as f64;

            if !u.is_finite() || !v.is_finite() || !w.is_finite() {
                Self::reset_particle(particle);
                continue;
            }

            // Nearest-cell temperature anomaly. This is a visual diagnostic, not a
            // substitute for the buoyancy term in the momentum equation.
            if !temperature.is_empty() {
                let ix = nearest_axis_index(coordinates.longitude(), particle.longitude);
                let iy = nearest_axis_index(coordinates.latitude(), particle.latitude);
                let iz = coordinates.depth().iter().enumerate()
                    .min_by(|(_, a), (_, b)| (**a - particle.depth).abs().total_cmp(&(**b - particle.depth).abs()))
                    .map(|(i, _)| i).unwrap_or(0);
                let nx = coordinates.width(); let ny = coordinates.height();
                let idx = iz.saturating_mul(nx * ny).saturating_add(iy.saturating_mul(nx)).saturating_add(ix);
                let t = temperature.get(idx).copied().filter(|value| value.is_finite()).unwrap_or(temperature_reference);
                // Use the domain mean rather than a hard-coded 10 C baseline so
                // both warm and cold local anomalies remain visible in other seasons.
                let anomaly = ((t - temperature_reference) / 4.0).clamp(-1.0, 1.0);
                particle.flow_color = if w > 0.015 {
                    [0.15, 1.0, 0.45] // local upwelling
                } else if w < -0.015 {
                    [0.95, 0.35, 0.12] // downwelling
                } else if anomaly >= 0.0 {
                    [1.0, 0.35 + 0.45 * (1.0 - anomaly), 0.08]
                } else {
                    [0.10, 0.45 + 0.35 * (1.0 + anomaly), 1.0]
                };
            } else {
                // Turning temperature visualization off restores a neutral flow color.
                particle.flow_color = FlowVertex::CYAN;
            }

            if u.hypot(v).hypot(w) < 0.02 {
                continue;
            }

            let east_m = u * dt as f64;
            let north_m = v * dt as f64;

            let delta_lat = north_m / 110_540.0;

            let cos_lat = particle.latitude.to_radians().cos();

            if cos_lat.abs() < 1.0e-6 {
                Self::reset_particle(particle);
                continue;
            }

            let delta_lon = east_m / (111_320.0 * cos_lat);

            let next_longitude = particle.longitude + delta_lon;

            let next_latitude = particle.latitude + delta_lat;

            // 深度は海面0、下向きを正とする。
            let next_depth = particle.depth - (w as f32) * dt;
            let next_position_is_valid = next_depth.is_finite()
                && coordinates.contains(next_longitude, next_latitude, next_depth)
                && next_longitude >= bounds.lon_min
                && next_longitude <= bounds.lon_max
                && next_latitude >= bounds.lat_min
                && next_latitude <= bounds.lat_max;

            if !next_position_is_valid {
                Self::reset_particle(particle);
                continue;
            }

            let mut world_start =
                converter.ocean_to_world(particle.longitude, particle.latitude, particle.depth);

            let mut world_end = converter.ocean_to_world(next_longitude, next_latitude, next_depth);

            // 海面をY=0、深度を下向きにし、海底地形と同じ垂直倍率で表示する。
            world_start[1] = -particle.depth * self.vertical_scale;
            world_end[1] = -next_depth * self.vertical_scale;

            if particle.trail.is_empty() {
                particle.trail.push(world_start);
            }

            particle.trail.push(world_end);

            if particle.trail.len() > self.max_trail_points {
                let excess = particle.trail.len() - self.max_trail_points;

                particle.trail.drain(0..excess);
            }

            particle.longitude = next_longitude;
            particle.latitude = next_latitude;
            particle.depth = next_depth;
        }
    }

    /// 粒子を初期位置に戻し、古い軌跡を消す。
    fn reset_particle(particle: &mut FlowParticle) {
        particle.longitude = particle.origin[0];
        particle.latitude = particle.origin[1];
        particle.depth = particle.origin_depth;

        particle.trail.clear();
    }

    /// 軌跡と現在位置のマーカーを描画用頂点に変換する。
    ///
    /// 既存のLineListパイプラインで描画できるように、
    /// 粒子は小さな十字形の線分として表現する。
    pub fn vertices(&self) -> Vec<FlowVertex> {
        let mut vertices = Vec::new();

        // ワールド座標単位でのマーカーの半径。
        let marker_size = 450.0_f32;

        for particle in &self.particles {
            // ------------------------------------------
            // 流線
            // ------------------------------------------

            for segment in particle.trail.windows(2) {
                vertices.push(FlowVertex {
                    position: segment[0],
                    color: particle.flow_color,
                });

                vertices.push(FlowVertex {
                    position: segment[1],
                    color: particle.flow_color,
                });
            }

            // ------------------------------------------
            // 粒子マーカー
            // ------------------------------------------

            let Some(&position) = particle.trail.last() else {
                continue;
            };

            let x = position[0];
            let y = position[1];
            let z = position[2];

            // X方向の線分。
            vertices.push(FlowVertex {
                position: [x - marker_size, y, z],
                    color: particle.flow_color,
            });

            vertices.push(FlowVertex {
                position: [x + marker_size, y, z],
                    color: particle.flow_color,
            });

            // Y方向の線分。
            vertices.push(FlowVertex {
                position: [x, y - marker_size, z],
                    color: particle.flow_color,
            });

            vertices.push(FlowVertex {
                position: [x, y + marker_size, z],
                    color: particle.flow_color,
            });

            // Z方向の線分。
            vertices.push(FlowVertex {
                position: [x, y, z - marker_size],
                    color: particle.flow_color,
            });

            vertices.push(FlowVertex {
                position: [x, y, z + marker_size],
                    color: particle.flow_color,
            });
        }

        vertices
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

fn nearest_axis_index(axis: &[f64], value: f64) -> usize {
    axis.iter().enumerate().min_by(|(_, a), (_, b)| (**a - value).abs().total_cmp(&(**b - value).abs())).map(|(i, _)| i).unwrap_or(0)
}
