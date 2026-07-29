#include <metal_stdlib>

using namespace metal;


struct GridInfo
{
    uint nx;
    uint ny;
    uint nz;

    float dx;
    float dy;
    float dz;
};


inline uint index3d(
    uint x,
    uint y,
    uint z,
    constant GridInfo& grid
)
{
    return
        z * grid.nx * grid.ny
        +
        y * grid.nx
        +
        x;
}


kernel void divergence_step(

    device const float* u
        [[buffer(0)]],

    device const float* v
        [[buffer(1)]],

    device const float* w
        [[buffer(2)]],


    device float* output
        [[buffer(3)]],


    constant GridInfo& grid
        [[buffer(4)]],


    uint3 id
        [[thread_position_in_grid]]

)
{

    uint x=id.x;
    uint y=id.y;
    uint z=id.z;


    if(
        x>=grid.nx ||
        y>=grid.ny ||
        z>=grid.nz
    )
    {
        return;
    }


    uint i =
        index3d(
            x,
            y,
            z,
            grid
        );


    //
    // boundary
    //
    if(
        x==0 ||
        y==0 ||
        z==0 ||
        x==grid.nx-1 ||
        y==grid.ny-1 ||
        z==grid.nz-1
    )
    {
        output[i]=0.0;
        return;
    }


    uint ix1 =
        index3d(
            x+1,
            y,
            z,
            grid
        );

    uint ix0 =
        index3d(
            x-1,
            y,
            z,
            grid
        );


    uint iy1 =
        index3d(
            x,
            y+1,
            z,
            grid
        );

    uint iy0 =
        index3d(
            x,
            y-1,
            z,
            grid
        );


    uint iz1 =
        index3d(
            x,
            y,
            z+1,
            grid
        );

    uint iz0 =
        index3d(
            x,
            y,
            z-1,
            grid
        );


    float du_dx =
        (u[ix1]-u[ix0])
        /
        (2.0 * grid.dx);


    float dv_dy =
        (v[iy1]-v[iy0])
        /
        (2.0 * grid.dy);


    float dw_dz =
        (w[iz1]-w[iz0])
        /
        (2.0 * grid.dz);



    output[i]
        =
        du_dx
        +
        dv_dy
        +
        dw_dz;
}
