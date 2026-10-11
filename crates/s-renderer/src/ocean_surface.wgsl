struct Camera {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) height: f32,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.view_proj * vec4<f32>(input.position, 1.0);
    return out;
}
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // Alpha 0.30: seabed and terrain remain visible beneath the ocean surface.
    return vec4<f32>(0.02, 0.42, 0.68, 0.30);
}
