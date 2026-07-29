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




kernel void derivative_z(

    device const float* input
        [[buffer(0)]],

    device float* output
        [[buffer(1)]],

    constant GridInfo& grid
        [[buffer(2)]],

    uint id
        [[thread_position_in_grid]]

)
{


    uint total =
        grid.nx
        *
        grid.ny
        *
        grid.nz;



    if(id >= total)
    {
        return;
    }



    uint layer =
        grid.nx
        *
        grid.ny;



    uint z =
        id / layer;



    if(z == 0)
    {

        output[id]
            =
        (
            input[id+layer]
            -
            input[id]
        )
        /
        grid.dz;

    }


    else if(z == grid.nz-1)
    {

        output[id]
            =
        (
            input[id]
            -
            input[id-layer]
        )
        /
        grid.dz;

    }


    else
    {

        output[id]
            =
        (
            input[id+layer]
            -
            input[id-layer]
        )
        /
        (2.0 * grid.dz);

    }

}
