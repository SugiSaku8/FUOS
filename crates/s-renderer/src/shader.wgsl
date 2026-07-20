struct Camera {
    view_proj: mat4x4<f32>,
};


@group(0)
@binding(0)
var<uniform> camera: Camera;



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

    @location(1)
    height: f32,
};



@vertex
fn vs_main(
    input: VertexInput
) -> VertexOutput {


    var output: VertexOutput;


    output.position =
        camera.view_proj *
        vec4<f32>(
            input.position,
            1.0
        );


    output.normal =
        input.normal;


    output.height =
        input.position.y;


    return output;
}




@fragment
fn fs_main(
    input: VertexOutput
)
-> @location(0) vec4<f32>
{


    //
    // 簡易高度カラー
    //

    let h =
        clamp(
            input.height / 10.0,
            0.0,
            1.0
        );


    let sea =
        vec3<f32>(
            0.0,
            0.25,
            0.55
        );


    let land =
        vec3<f32>(
            0.15,
            0.65,
            0.25
        );


    let color =
        mix(
            sea,
            land,
            h
        );


    //
    // 簡易ライティング
    //

    let light =
        max(
            dot(
                normalize(input.normal),
                vec3<f32>(
                    0.3,
                    1.0,
                    0.5
                )
            ),
            0.2
        );


    return vec4<f32>(
        color * light,
        1.0
    );
}
