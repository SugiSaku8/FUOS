struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
};


@group(0)
@binding(0)
var<uniform> camera: CameraUniform;



struct VertexInput {

    @location(0)
    position: vec3<f32>,

    @location(1)
    normal: vec3<f32>,
};



struct VertexOutput {

    @builtin(position)
    position: vec4<f32>,


    @location(0)
    normal: vec3<f32>,
};



@vertex
fn vs_main(
    input: VertexInput
)
-> VertexOutput {

    var output: VertexOutput;


    output.position =
        camera.view_projection *
        vec4<f32>(
            input.position,
            1.0
        );


    output.normal =
        input.normal;


    return output;
}



@fragment
fn fs_main(
    input: VertexOutput
)
-> @location(0) vec4<f32> {


    let light =
        normalize(
            vec3<f32>(
                0.3,
                0.7,
                1.0
            )
        );


    let intensity =
        max(
            dot(
                normalize(input.normal),
                light
            ),
            0.0
        );


    let base =
        vec3<f32>(
            0.12,
            0.32,
            0.18
        );


    return vec4<f32>(
        base * intensity,
        1.0
    );
}
