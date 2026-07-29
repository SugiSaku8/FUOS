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



kernel void gradient_pressure(

    device const float* pressure
        [[buffer(0)]],


    device const float* u
        [[buffer(1)]],

    device const float* v
        [[buffer(2)]],

    device const float* w
        [[buffer(3)]],


    device float* out_u
        [[buffer(4)]],

    device float* out_v
        [[buffer(5)]],

    device float* out_w
        [[buffer(6)]],


    constant GridInfo& grid
        [[buffer(7)]],


    constant float& dt
        [[buffer(8)]],

    constant float& rho
        [[buffer(9)]],


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
        out_u[i]=u[i];
        out_v[i]=v[i];
        out_w[i]=w[i];

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



    float dp_dx =
        (pressure[ix1]-pressure[ix0])
        /
        (2.0*grid.dx);



    float dp_dy =
        (pressure[iy1]-pressure[iy0])
        /
        (2.0*grid.dy);



    float dp_dz =
        (pressure[iz1]-pressure[iz0])
        /
        (2.0*grid.dz);



    float factor =
        dt / rho;



    out_u[i]
        =
        u[i]
        -
        factor * dp_dx;



    out_v[i]
        =
        v[i]
        -
        factor * dp_dy;



    out_w[i]
        =
        w[i]
        -
        factor * dp_dz;

}
