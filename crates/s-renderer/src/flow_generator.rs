use s_field::VelocityField;

/// VelocityFieldから流線を生成する。
///
/// 現在はEuler積分。
pub struct FlowGenerator;

impl FlowGenerator {
    pub fn generate(
        velocity: &VelocityField,
        seeds: &[[f32; 3]],
        step: f32,
        length: usize,
    ) -> Vec<[f32; 3]> {
        let mut vertices = Vec::new();

        for seed in seeds {
            let mut p = *seed;

            for _ in 0..length {
                let vel = sample_velocity(velocity, p);

                let next = [
                    p[0] + vel[0] * step,
                    p[1] + vel[1] * step,
                    p[2] + vel[2] * step,
                ];

                //
                // LineList用
                //
                vertices.push(p);

                vertices.push(next);

                p = next;

                //
                // Grid外へ出たら終了
                //
                if !inside(velocity, p) {
                    break;
                }
            }
        }

        vertices
    }
}

fn sample_velocity(velocity: &VelocityField, position: [f32; 3]) -> [f32; 3] {
    let shape = velocity.shape();

    let x = position[0].clamp(0.0, (shape[0] - 1) as f32) as usize;

    let y = position[1].clamp(0.0, (shape[1] - 1) as f32) as usize;

    let z = position[2].clamp(0.0, (shape[2] - 1) as f32) as usize;

    let index = z * shape[0] * shape[1] + y * shape[0] + x;

    let u = velocity.u().time_slice(0).unwrap()[index];

    let v = velocity.v().time_slice(0).unwrap()[index];

    let w = velocity.w().time_slice(0).unwrap()[index];

    [u, v, w]
}

fn inside(velocity: &VelocityField, p: [f32; 3]) -> bool {
    let shape = velocity.shape();

    p[0] >= 0.0
        && p[1] >= 0.0
        && p[2] >= 0.0
        && p[0] < shape[0] as f32
        && p[1] < shape[1] as f32
        && p[2] < shape[2] as f32
}
