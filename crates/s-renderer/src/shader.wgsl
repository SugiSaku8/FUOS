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

    @location(2)
    height: f32,
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
)
-> VertexOutput {

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
        input.height;


    return output;
}




@fragment
fn fs_main(
    input: VertexOutput
)
-> @location(0) vec4<f32>
{

    let h = input.height;


    var color: vec3<f32>;


    //
    // 高度・水深カラー
    //
    // h > 0    陸地
    // h = 0    海面
    // h < 0    海底
    //


    if h < -30.0 {

        // 深海

        color =
            vec3<f32>(
                0.0,
                0.03,
                0.25
            );


    } else if h < -5.0 {

        // 中深度

        color =
            vec3<f32>(
                0.0,
                0.25,
                0.65
            );


    } else if h < 0.0 {

        // 浅瀬

        color =
            vec3<f32>(
                0.0,
                0.65,
                0.9
            );


    } else if h < 5.0 {

        // 海岸

        color =
            vec3<f32>(
                0.75,
                0.7,
                0.4
            );


    } else {

        // 陸地

        color =
            vec3<f32>(
                0.2,
                0.65,
                0.2
            );
    }



    //
    // 簡易ライティング
    //

    let light =
        max(
            dot(
                normalize(input.normal),
                normalize(
                    vec3<f32>(
                        0.3,
                        1.0,
                        0.5
                    )
                )
            ),
            0.25
        );



    return vec4<f32>(
        color * light,
        1.0
    );
}
