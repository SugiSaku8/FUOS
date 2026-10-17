struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) height: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) height: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.height = input.height;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let h = input.height;

    // 海面：透明感のある青色
    if h <= 0.0 {
        return vec4<f32>(0.025, 0.32, 0.58, 0.28);
    }

    // 陸地
    let land_t = clamp(h / 1000.0, 0.0, 1.0);
    let lowland = vec3<f32>(0.12, 0.25, 0.12);
    let highland = vec3<f32>(0.28, 0.30, 0.22);
    let land_color = mix(lowland, highland, land_t);

    return vec4<f32>(land_color, 1.0);
}
