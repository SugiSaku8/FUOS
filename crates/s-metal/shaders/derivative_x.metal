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




kernel void derivative_x(

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



    uint x =
        id
        %
        grid.nx;



    if(x == 0)
    {

        output[id]
            =
        (
            input[id+1]
            -
            input[id]
        )
        /
        grid.dx;

    }


    else if(x == grid.nx-1)
    {

        output[id]
            =
        (
            input[id]
            -
            input[id-1]
        )
        /
        grid.dx;

    }


    else
    {

        output[id]
            =
        (
            input[id+1]
            -
            input[id-1]
        )
        /
        (2.0 * grid.dx);

    }

}
