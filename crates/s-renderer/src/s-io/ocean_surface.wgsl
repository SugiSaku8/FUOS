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
fn vs_main(
    input: VertexInput
) -> VertexOutput {
    var output: VertexOutput;

    output.position =
        camera.view_proj
        * vec4<f32>(input.position, 1.0);

    return output;
}


@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // RGBは海面の色、Alphaは不透明度。
    // 0.30 = 不透明度30%、透過率70%。
    return vec4<f32>(
        0.02,
        0.42,
        0.68,
        0.30
    );
}
