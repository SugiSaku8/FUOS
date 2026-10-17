use crate::{coordinate::GeoConverter, flow::FlowVertex};

use s_field::VelocityField;

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

                // 既存の海流表示と同じ高さ補正。

                vertices.push(FlowVertex {
                    position: world_start,
                });

                vertices.push(FlowVertex {
                    position: world_end,
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
}

/// 流速データを使って粒子を時間発展させるシステム。
pub struct FlowParticleSystem {
    particles: Vec<FlowParticle>,
    max_trail_points: usize,
}

impl FlowParticleSystem {
    /// 初期位置から粒子を作成する。
    ///
    /// seedsは[経度、緯度、深度]。
    pub fn new(seeds: &[[f32; 3]], max_trail_points: usize) -> Self {
        let particles = seeds
            .iter()
            .map(|seed| FlowParticle {
                origin: [seed[0] as f64, seed[1] as f64],
                origin_depth: seed[2],

                longitude: seed[0] as f64,
                latitude: seed[1] as f64,
                depth: seed[2],

                trail: Vec::new(),
            })
            .collect();

        Self {
            particles,
            max_trail_points: max_trail_points.max(2),
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
        let Some(coordinates) = velocity.coordinates() else {
            return;
        };

        let bounds = converter.bounds();

        // speed_multiplier はシミュレーション時間の倍率。流速値そのものは変更しない。
        let dt = dt_seconds.clamp(0.0, 0.05) * speed_multiplier.max(0.0);

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

            // 既存の海流表示と同じ高さ補正。

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

    /// 各粒子の軌跡をGPU描画用の線分に変換する。
    pub fn vertices(&self) -> Vec<FlowVertex> {
        let mut vertices = Vec::new();

        for particle in &self.particles {
            for segment in particle.trail.windows(2) {
                vertices.push(FlowVertex {
                    position: segment[0],
                });
                vertices.push(FlowVertex {
                    position: segment[1],
                });
            }
        }

        vertices
    }

    /// 現在の粒子位置を、3次元空間で視認しやすいマーカー頂点に変換する。
    /// 粒子の最終軌跡点を使うため、流線と同じ水深・座標を保つ。
    pub fn particle_vertices(&self) -> Vec<FlowVertex> {
        const MARKER_RADIUS: f32 = 450.0;
        let mut vertices = Vec::with_capacity(self.particles.len() * 6);

        for particle in &self.particles {
            let Some(&position) = particle.trail.last() else {
                continue;
            };
            let [x, y, z] = position;
            vertices.push(FlowVertex {
                position: [x - MARKER_RADIUS, y, z],
            });
            vertices.push(FlowVertex {
                position: [x + MARKER_RADIUS, y, z],
            });
            vertices.push(FlowVertex {
                position: [x, y - MARKER_RADIUS, z],
            });
            vertices.push(FlowVertex {
                position: [x, y + MARKER_RADIUS, z],
            });
            vertices.push(FlowVertex {
                position: [x, y, z - MARKER_RADIUS],
            });
            vertices.push(FlowVertex {
                position: [x, y, z + MARKER_RADIUS],
            });
        }

        vertices
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}
