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




kernel void derivative_y(

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



    uint y =
        (id / grid.nx)
        %
        grid.ny;



    uint stride =
        grid.nx;



    if(y == 0)
    {

        output[id]
            =
        (
            input[id + stride]
            -
            input[id]
        )
        /
        grid.dy;

    }


    else if(y == grid.ny-1)
    {

        output[id]
            =
        (
            input[id]
            -
            input[id - stride]
        )
        /
        grid.dy;

    }


    else
    {

        output[id]
            =
        (
            input[id + stride]
            -
            input[id - stride]
        )
        /
        (2.0 * grid.dy);

    }


}
