use crate::{coordinate::GeoConverter, flow::FlowVertex};
use s_field::VelocityField;

/// 静的な流線を生成する。
pub struct FlowGenerator;

impl FlowGenerator {
    /// 流速データから流線を生成する。
    ///
    /// # Arguments
    ///
    /// * `velocity` - 海流の速度場
    /// * `converter` - 地理座標とワールド座標の変換器
    /// * `seeds` - 粒子の初期位置 [経度, 緯度, 深度]
    /// * `dt_seconds` - 1ステップあたりの時間
    /// * `length` - 最大ステップ数
    /// * `time` - 速度場の時間座標
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

                // 東西・南北方向の移動距離。
                let east_m = u * dt_seconds as f64;
                let north_m = v * dt_seconds as f64;

                // 距離を緯度・経度の変化量に変換。
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

                // 海面上に描画するためのオフセット。
                world_start[1] += 100.0;
                world_end[1] += 100.0;

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

/// 1つの流速粒子と、その粒子が通過した位置を保持する。
struct FlowParticle {
    origin: [f64; 2],
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
    /// `seeds` は [経度, 緯度, 深度] の配列。
    pub fn new(seeds: &[[f32; 3]], max_trail_points: usize) -> Self {
        let particles = seeds
            .iter()
            .map(|seed| FlowParticle {
                origin: [seed[0] as f64, seed[1] as f64],
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
    ///
    /// `dt_seconds` は実際のフレーム経過時間。
    /// `speed_multiplier` はシミュレーション速度の倍率。
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

        // フレームレートが落ちた場合の急激な移動を防ぐ。
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

            // 領域外に出た粒子は初期位置に戻す。
            if !position_is_valid {
                Self::reset_particle(particle);
                continue;
            }

            let Some(vel) =
                velocity.sample(particle.longitude, particle.latitude, particle.depth, time)
            else {
                Self::reset_particle(particle);
                continue;
            };

            let u = vel[0] as f64;
            let v = vel[1] as f64;

            if !u.is_finite() || !v.is_finite() {
                Self::reset_particle(particle);
                continue;
            }

            // 静止に近い粒子は、その場に残す。
            if u.hypot(v) < 0.02 {
                continue;
            }

            // 流速[m/s] × 経過時間[s] = 移動距離[m]。
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

            let next_position_is_valid =
                coordinates.contains(next_longitude, next_latitude, particle.depth)
                    && next_longitude >= bounds.lon_min
                    && next_longitude <= bounds.lon_max
                    && next_latitude >= bounds.lat_min
                    && next_latitude <= bounds.lat_max;

            if !next_position_is_valid {
                Self::reset_particle(particle);
                continue;
            }

            // 移動前の位置も軌跡に加える。
            // 最初のフレームから線分を生成できるようにする。
            let mut world_start =
                converter.ocean_to_world(particle.longitude, particle.latitude, particle.depth);

            world_start[1] += 100.0;

            let mut world_end =
                converter.ocean_to_world(next_longitude, next_latitude, particle.depth);

            world_end[1] += 100.0;

            if particle.trail.is_empty() {
                particle.trail.push(world_start);
            }

            particle.trail.push(world_end);

            // 軌跡の長さを制限する。
            if particle.trail.len() > self.max_trail_points {
                let excess = particle.trail.len() - self.max_trail_points;

                particle.trail.drain(0..excess);
            }

            particle.longitude = next_longitude;
            particle.latitude = next_latitude;
        }
    }

    /// 粒子を初期位置に戻し、古い軌跡を消す。
    fn reset_particle(particle: &mut FlowParticle) {
        particle.longitude = particle.origin[0];
        particle.latitude = particle.origin[1];
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

    /// 粒子の数を返す。
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}
