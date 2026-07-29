#include <metal_stdlib>

using namespace metal;


struct GridInfo {

    uint nx;
    uint ny;
    uint nz;

    float dx;
    float dy;
    float dz;

};



kernel void advection(

    device const float* u
        [[buffer(0)]],

    device const float* v
        [[buffer(1)]],

    device const float* w
        [[buffer(2)]],


    device const float* dudx
        [[buffer(3)]],

    device const float* dudy
        [[buffer(4)]],

    device const float* dudz
        [[buffer(5)]],


    device float* output
        [[buffer(6)]],


    constant GridInfo& grid
        [[buffer(7)]],


    uint id
        [[thread_position_in_grid]]

)
{


    uint total =
        grid.nx *
        grid.ny *
        grid.nz;


    if(id >= total)
    {
        return;
    }



    output[id] =
        -(
            u[id] * dudx[id]
          + v[id] * dudy[id]
          + w[id] * dudz[id]
        );


}
