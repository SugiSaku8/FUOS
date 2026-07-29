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
    constant GridInfo& g
)
{
    return
        z*g.nx*g.ny
        +
        y*g.nx
        +
        x;
}




kernel void pressure_jacobi(

    device const float* p
        [[buffer(0)]],

    device const float* div
        [[buffer(1)]],


    device float* out
        [[buffer(2)]],


    constant GridInfo& grid
        [[buffer(3)]],


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
        return;



    uint i =
        index3d(
            x,y,z,grid
        );



    //
    // boundary pressure = 0
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

        out[i]=0.0;

        return;

    }




    uint xm =
        index3d(
            x-1,y,z,grid
        );

    uint xp =
        index3d(
            x+1,y,z,grid
        );


    uint ym =
        index3d(
            x,y-1,z,grid
        );


    uint yp =
        index3d(
            x,y+1,z,grid
        );


    uint zm =
        index3d(
            x,y,z-1,grid
        );


    uint zp =
        index3d(
            x,y,z+1,grid
        );




    float laplace_neighbors =
        p[xm]
        +
        p[xp]
        +
        p[ym]
        +
        p[yp]
        +
        p[zm]
        +
        p[zp];



    float h2 =
        grid.dx
        *
        grid.dx;



    //
    // Jacobi iteration
    //
    // p_new =
    // (neighbors - div*h²)/6
    //

    out[i] =
        (
            laplace_neighbors
            -
            div[i]*h2
        )
        /
        6.0;

}
