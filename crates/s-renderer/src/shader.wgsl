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
    height: f32,
};



struct VertexOutput {

    @builtin(position)
    position: vec4<f32>,

    @location(0)
    height: f32,
};



@vertex
fn vs_main(
    input: VertexInput
) -> VertexOutput {

    var out: VertexOutput;


    out.position =
        camera.view_proj *
        vec4<f32>(
            input.position,
            1.0
        );


    out.height = input.height;


    return out;
}


@fragment
fn fs_main(
    input: VertexOutput
)
-> @location(0) vec4<f32>
{

    let h = input.height;


    // 深海
    if h < -2000.0 {

        return vec4<f32>(
            0.0,
            0.05,
            0.4,
            1.0
        );
    }


    // 浅海
    if h < -500.0 {

        return vec4<f32>(
            0.0,
            0.4,
            0.8,
            1.0
        );
    }


    // 海岸
    if h < 0.0 {

        return vec4<f32>(
            0.8,
            0.8,
            0.4,
            1.0
        );
    }


    // 陸地
    if h < 500.0 {

        return vec4<f32>(
            0.1,
            0.6,
            0.15,
            1.0
        );
    }


    // 山
    return vec4<f32>(
        0.8,
        0.8,
        0.8,
        1.0
    );
}
