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

    // 海底：浅い場所から深い場所へ暗くする。
    if h < 0.0 {
        let depth = -h;

        // 0～4000 mの水深を0～1に正規化。
        let d = clamp(depth / 4000.0, 0.0, 1.0);

        // 浅い海底は青、深海は濃紺。
        // 浅い海：透明感のある青緑

        let shallow = vec3<f32>(0.055, 0.29, 0.34);

        // 深い海：落ち着いた深い藍色

        let deep = vec3<f32>(0.012, 0.075, 0.19);

        // 深くなるほど暗くし、海底の起伏を読み取りやすくする。
        let color = mix(shallow, deep, pow(d, 0.75));

        return vec4<f32>(color, 1.0);
    }

    // 陸地：海底との境界を明確にする。
    let land_t = clamp(h / 1000.0, 0.0, 1.0);
    let lowland = vec3<f32>(0.12, 0.25, 0.12);
    let highland = vec3<f32>(0.28, 0.30, 0.22);
    let land_color = mix(lowland, highland, land_t);

    return vec4<f32>(land_color, 1.0);
}
