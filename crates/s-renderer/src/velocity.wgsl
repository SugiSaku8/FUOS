/*
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
    velocity: vec3<f32>,
};



struct VertexOutput {

    @builtin(position)
    position: vec4<f32>,
};



@vertex
fn vs_main(
    input: VertexInput,
    @builtin(vertex_index) vertex_index: u32
)->VertexOutput {


    var out: VertexOutput;


    let scale = 0.02;


    var pos = input.position;


    //
    // 2頂点のラインを作る
    //
    if vertex_index == 1u {

        pos =
            input.position +
            input.velocity * scale;

    }


    out.position =
        camera.view_proj *
        vec4<f32>(
            pos,
            1.0
        );


    return out;
}



@fragment
fn fs_main()
    -> @location(0) vec4<f32>
{

    return vec4<f32>(
        1.0,
        0.1,
        0.1,
        1.0
    );

}
*/

struct Camera {
    view_proj: mat4x4<f32>,
};


@group(0)
@binding(0)
var<uniform> camera: Camera;



struct VertexInput {

    @location(0)
    position: vec3<f32>,

};



struct VertexOutput {

    @builtin(position)
    position: vec4<f32>,

};



@vertex
fn vs_main(
    input: VertexInput
)->VertexOutput {

    var out: VertexOutput;


    out.position =
        camera.view_proj *
        vec4<f32>(
            input.position,
            1.0
        );


    return out;
}



@fragment
fn fs_main()
    -> @location(0) vec4<f32>
{

    return vec4<f32>(
        0.0,
        0.5,
        1.0,
        1.0
    );

}
