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



kernel void diffusion_step(

    device const float* input
        [[buffer(0)]],

    device float* output
        [[buffer(1)]],

    constant GridInfo& grid
        [[buffer(2)]],

    constant float& viscosity_horizontal [[buffer(3)]],
    constant float& viscosity_vertical [[buffer(4)]],
    constant float& dt [[buffer(5)]],

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
        output[i]=input[i];
        return;
    }



    uint xp=index3d(x+1,y,z,grid);
    uint xm=index3d(x-1,y,z,grid);

    uint yp=index3d(x,y+1,z,grid);
    uint ym=index3d(x,y-1,z,grid);

    uint zp=index3d(x,y,z+1,grid);
    uint zm=index3d(x,y,z-1,grid);



    float laplacian_horizontal =
        (input[xp] + input[xm] - 2.0*input[i])/(grid.dx*grid.dx)
        + (input[yp] + input[ym] - 2.0*input[i])/(grid.dy*grid.dy);
    float laplacian_vertical =
        (input[zp] + input[zm] - 2.0*input[i])/(grid.dz*grid.dz);

    output[i] = input[i] + dt *
        (viscosity_horizontal * laplacian_horizontal + viscosity_vertical * laplacian_vertical);

}
