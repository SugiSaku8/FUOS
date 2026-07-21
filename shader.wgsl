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
    @location(0) height: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.view_proj * vec4<f32>(input.position,1.0);
    out.height = input.height;
    return out;
}

fn ocean_color(depth:f32)->vec3<f32>{
    if(depth<50.0){return vec3<f32>(0.15,0.75,0.85);}
    else if(depth<100.0){return vec3<f32>(0.05,0.65,0.80);}
    else if(depth<150.0){return vec3<f32>(0.02,0.55,0.75);}
    else if(depth<200.0){return vec3<f32>(0.00,0.45,0.70);}
    else if(depth<225.0){return vec3<f32>(0.00,0.42,0.68);}
    else if(depth<250.0){return vec3<f32>(0.00,0.39,0.66);}
    else if(depth<275.0){return vec3<f32>(0.00,0.36,0.63);}
    else if(depth<300.0){return vec3<f32>(0.00,0.33,0.60);}
    else if(depth<400.0){return vec3<f32>(0.00,0.29,0.54);}
    else if(depth<500.0){return vec3<f32>(0.00,0.25,0.48);}
    else if(depth<600.0){return vec3<f32>(0.00,0.21,0.43);}
    else if(depth<700.0){return vec3<f32>(0.00,0.17,0.37);}
    else if(depth<800.0){return vec3<f32>(0.00,0.14,0.32);}
    else if(depth<900.0){return vec3<f32>(0.00,0.11,0.28);}
    else if(depth<1000.0){return vec3<f32>(0.00,0.09,0.24);}
    else if(depth<1500.0){return vec3<f32>(0.00,0.07,0.18);}
    else if(depth<2000.0){return vec3<f32>(0.00,0.05,0.14);}
    else if(depth<2500.0){return vec3<f32>(0.00,0.03,0.10);}
    return vec3<f32>(0.00,0.01,0.06);
}

fn land_color(h:f32)->vec3<f32>{
    if(h<50.0){return vec3<f32>(0.85,0.80,0.55);}
    else if(h<100.0){return vec3<f32>(0.75,0.82,0.45);}
    else if(h<150.0){return vec3<f32>(0.60,0.75,0.35);}
    else if(h<200.0){return vec3<f32>(0.45,0.70,0.28);}
    else if(h<250.0){return vec3<f32>(0.35,0.65,0.22);}
    else if(h<300.0){return vec3<f32>(0.28,0.60,0.18);}
    else if(h<400.0){return vec3<f32>(0.22,0.55,0.16);}
    else if(h<500.0){return vec3<f32>(0.18,0.50,0.14);}
    else if(h<700.0){return vec3<f32>(0.30,0.45,0.18);}
    else if(h<900.0){return vec3<f32>(0.40,0.40,0.20);}
    else if(h<1100.0){return vec3<f32>(0.50,0.42,0.25);}
    else if(h<1300.0){return vec3<f32>(0.60,0.50,0.35);}
    else if(h<1600.0){return vec3<f32>(0.70,0.62,0.48);}
    else if(h<2000.0){return vec3<f32>(0.82,0.78,0.70);}
    else if(h<2500.0){return vec3<f32>(0.90,0.88,0.84);}
    else if(h<3000.0){return vec3<f32>(0.95,0.95,0.95);}
    return vec3<f32>(1.0,1.0,1.0);
}

@fragment
fn fs_main(input:VertexOutput)->@location(0) vec4<f32>{
    if(input.height<0.0){
        return vec4<f32>(ocean_color(-input.height),1.0);
    }
    return vec4<f32>(land_color(input.height),1.0);
}
