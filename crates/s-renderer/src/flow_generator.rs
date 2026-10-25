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

                // `max_trail_points` is an initial allocation hint only; trails
                // grow until the configured speed threshold clears them.
                trail: Vec::with_capacity(max_trail_points.max(2)),
                flow_color: FlowVertex::CYAN,
            })
            .collect();

        Self {
            particles,
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
        self.update_with_temperature_and_disappear_speed(velocity, &[], converter, dt_seconds, speed_multiplier, time, 0.02);
    }

    pub fn update_with_disappear_speed(
        &mut self,
        velocity: &VelocityField,
        converter: &GeoConverter,
        dt_seconds: f32,
        speed_multiplier: f32,
        time: f64,
        disappear_speed_mps: f32,
    ) {
        self.update_with_temperature_and_disappear_speed(
            velocity, &[], converter, dt_seconds, speed_multiplier, time, disappear_speed_mps,
        );
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
        self.update_with_temperature_and_disappear_speed(
            velocity, temperature, converter, dt_seconds, speed_multiplier, time, 0.02,
        );
    }

    /// The trail grows continuously. It is cleared only when local 3-D speed
    /// reaches or falls below `disappear_speed_mps` (or the particle exits the domain).
    pub fn update_with_temperature_and_disappear_speed(
        &mut self,
        velocity: &VelocityField,
        temperature: &[f32],
        converter: &GeoConverter,
        dt_seconds: f32,
        speed_multiplier: f32,
        time: f64,
        disappear_speed_mps: f32,
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

        // 視覚化用の時間倍率。倍率を完全に外すと広域の海流では移動が
        // 見えにくくなるため、加速は維持しつつ、下で格子幅に応じて制限する。
        let base_dt = dt_seconds.clamp(0.0, 0.05) * speed_multiplier.max(0.0) * 10.0;

        if !base_dt.is_finite() || base_dt <= 0.0 {
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

            let local_speed = u.hypot(v).hypot(w);
            let disappear_threshold = if disappear_speed_mps.is_finite() {
                disappear_speed_mps.max(0.0) as f64
            } else { 0.02 };
            if local_speed <= disappear_threshold {
                // Do not keep a stale trail when the flow slows to the configured
                // threshold. The particle remains at its current location; if the
                // current later accelerates, a new trail begins there.
                particle.trail.clear();
                continue;
            }

            // 1フレームで格子幅を飛び越えないように、水平・鉛直成分ごとに
            // 移動時間を制限する。これにより領域外判定→軌跡全消去の連発を防ぐ。
            let cos_lat_abs = particle.latitude.to_radians().cos().abs().max(1.0e-6);
            let min_lon_step = coordinates.longitude().windows(2)
                .map(|pair| (pair[1] - pair[0]).abs()).fold(f64::INFINITY, f64::min);
            let min_lat_step = coordinates.latitude().windows(2)
                .map(|pair| (pair[1] - pair[0]).abs()).fold(f64::INFINITY, f64::min);
            let min_depth_step = coordinates.depth().windows(2)
                .map(|pair| (pair[1] - pair[0]).abs() as f64).fold(f64::INFINITY, f64::min);

            let horizontal_cell_m = (min_lon_step * 111_320.0 * cos_lat_abs)
                .min(min_lat_step * 110_540.0);
            let vertical_cell_m = min_depth_step;
            let horizontal_speed = u.hypot(v);
            let mut dt = base_dt as f64;
            if horizontal_cell_m.is_finite() && horizontal_cell_m > 0.0 && horizontal_speed > 1.0e-9 {
                dt = dt.min(horizontal_cell_m * 0.20 / horizontal_speed);
            }
            if vertical_cell_m.is_finite() && vertical_cell_m > 0.0 && w.abs() > 1.0e-9 {
                dt = dt.min(vertical_cell_m * 0.20 / w.abs());
            }
            if !dt.is_finite() || dt <= 0.0 {
                continue;
            }

            let east_m = u * dt;
            let north_m = v * dt;

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
            let next_depth = particle.depth - (w * dt) as f32;
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
