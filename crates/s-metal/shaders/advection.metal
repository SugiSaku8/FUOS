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



kernel void advection_step(

    device const float* u
        [[buffer(0)]],

    device const float* v
        [[buffer(1)]],

    device const float* w
        [[buffer(2)]],


    device float* out_u
        [[buffer(3)]],

    device float* out_v
        [[buffer(4)]],

    device float* out_w
        [[buffer(5)]],


    constant GridInfo& grid
        [[buffer(6)]],


    constant float& dt
        [[buffer(7)]],


    uint3 id
        [[thread_position_in_grid]]

)
{

    uint x = id.x;
    uint y = id.y;
    uint z = id.z;


    if(
        x >= grid.nx ||
        y >= grid.ny ||
        z >= grid.nz
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
    // boundary condition
    //
    if(
        x == 0 ||
        y == 0 ||
        z == 0 ||
        x == grid.nx - 1 ||
        y == grid.ny - 1 ||
        z == grid.nz - 1
    )
    {
        out_u[i] = u[i];
        out_v[i] = v[i];
        out_w[i] = w[i];

        return;
    }



    //
    // neighbor index
    //

    uint ix1 =
        index3d(
            x + 1,
            y,
            z,
            grid
        );

    uint ix0 =
        index3d(
            x - 1,
            y,
            z,
            grid
        );


    uint iy1 =
        index3d(
            x,
            y + 1,
            z,
            grid
        );

    uint iy0 =
        index3d(
            x,
            y - 1,
            z,
            grid
        );


    uint iz1 =
        index3d(
            x,
            y,
            z + 1,
            grid
        );

    uint iz0 =
        index3d(
            x,
            y,
            z - 1,
            grid
        );



    //
    // central difference
    //

    float du_dx =
        (u[ix1] - u[ix0])
        /
        (2.0 * grid.dx);


    float du_dy =
        (u[iy1] - u[iy0])
        /
        (2.0 * grid.dy);


    float du_dz =
        (u[iz1] - u[iz0])
        /
        (2.0 * grid.dz);



    float dv_dx =
        (v[ix1] - v[ix0])
        /
        (2.0 * grid.dx);


    float dv_dy =
        (v[iy1] - v[iy0])
        /
        (2.0 * grid.dy);


    float dv_dz =
        (v[iz1] - v[iz0])
        /
        (2.0 * grid.dz);



    float dw_dx =
        (w[ix1] - w[ix0])
        /
        (2.0 * grid.dx);


    float dw_dy =
        (w[iy1] - w[iy0])
        /
        (2.0 * grid.dy);


    float dw_dz =
        (w[iz1] - w[iz0])
        /
        (2.0 * grid.dz);



    float uu = u[i];
    float vv = v[i];
    float ww = w[i];



    //
    // (u · ∇)u
    //

    float adv_u =
        uu * du_dx
        +
        vv * du_dy
        +
        ww * du_dz;



    float adv_v =
        uu * dv_dx
        +
        vv * dv_dy
        +
        ww * dv_dz;



    float adv_w =
        uu * dw_dx
        +
        vv * dw_dy
        +
        ww * dw_dz;



    //
    // Euler forward
    //

    out_u[i] =
        uu
        -
        dt * adv_u;


    out_v[i] =
        vv
        -
        dt * adv_v;


    out_w[i] =
        ww
        -
        dt * adv_w;

}
