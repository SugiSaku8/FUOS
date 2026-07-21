struct Camera {

    view_proj:
        mat4x4<f32>,

};



@group(0)
@binding(0)
var<uniform> camera:Camera;



struct VertexInput {


    @location(0)
    position:vec3<f32>,


    @location(1)
    height:f32,

};



struct VertexOutput {


    @builtin(position)
    position:vec4<f32>,


    @location(0)
    height:f32,

};



@vertex
fn vs_main(
    input:VertexInput
)
->VertexOutput
{


    var out:VertexOutput;


    out.position =
        camera.view_proj *
        vec4<f32>(
            input.position,
            1.0
        );


    out.height =
        input.height;


    return out;

}




//
// 駿河湾 GEBCO カラーマップ
//
fn terrain_color(
    h:f32
)
->vec3<f32>
{


    //
    // 深海
    //
    if h < -3000.0 {

        return vec3<f32>(
            0.01,
            0.02,
            0.12
        );

    }



    //
    // -3000 ～ -2000
    //
    if h < -2000.0 {


        let t =
            (h+3000.0)/1000.0;


        return mix(
            vec3<f32>(
                0.01,
                0.02,
                0.12
            ),

            vec3<f32>(
                0.0,
                0.12,
                0.45
            ),

            t
        );

    }



    //
    // -2000 ～ -1000
    //
    if h < -1000.0 {


        let t =
            (h+2000.0)/1000.0;


        return mix(
            vec3<f32>(
                0.0,
                0.12,
                0.45
            ),

            vec3<f32>(
                0.0,
                0.5,
                0.85
            ),

            t
        );

    }



    //
    // 浅海
    //
    if h < -100.0 {


        let t =
            (h+1000.0)/900.0;


        return mix(
            vec3<f32>(
                0.0,
                0.5,
                0.85
            ),

            vec3<f32>(
                0.3,
                0.8,
                0.9
            ),

            t
        );

    }



    //
    // 海岸
    //
    if h < 0.0 {

        return vec3<f32>(
            0.85,
            0.8,
            0.45
        );

    }



    //
    // 平野
    //
    if h < 500.0 {


        let t=h/500.0;


        return mix(
            vec3<f32>(
                0.25,
                0.65,
                0.2
            ),

            vec3<f32>(
                0.45,
                0.32,
                0.15
            ),

            t
        );

    }



    //
    // 山岳
    //
    if h < 2000.0 {


        let t =
            (h-500.0)/1500.0;


        return mix(

            vec3<f32>(
                0.45,
                0.32,
                0.15
            ),

            vec3<f32>(
                0.9,
                0.9,
                0.9
            ),

            t
        );

    }



    //
    // 富士山頂
    //
    return vec3<f32>(
        1.0,
        1.0,
        1.0
    );

}





@fragment
fn fs_main(
    input:VertexOutput
)
->@location(0) vec4<f32>
{


    let color =
        terrain_color(
            input.height
        );


    return vec4<f32>(
        color,
        1.0
    );

}
