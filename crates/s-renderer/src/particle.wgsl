struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) depth: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.position = clip_position;
    out.depth = clamp(clip_position.z / max(clip_position.w, 0.0001), 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 粒子は白。遠方は薄くし、シアンの流跡と瞬間位置を区別する。
    let alpha = mix(1.0, 0.55, input.depth);
    return vec4<f32>(1.0, 1.0, 1.0, alpha);
}
