use glam::Vec3;

pub struct OceanParticle {
    pub position: Vec3,

    pub velocity: Vec3,
}

pub struct ParticleSystem {
    pub particles: Vec<OceanParticle>,
}

impl ParticleSystem {
    pub fn new(count: usize) -> Self {
        let mut particles = Vec::with_capacity(count);

        for i in 0..count {
            let x = (i % 128) as f32;

            let y = ((i / 128) % 128) as f32;

            let z = (i / (128 * 128)) as f32;

            particles.push(OceanParticle {
                position: Vec3::new(x, y, -z),

                velocity: Vec3::ZERO,
            });
        }

        Self { particles }
    }

    pub fn advect(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.position += p.velocity * dt;
        }
    }
}
