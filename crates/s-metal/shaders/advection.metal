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

    // Depth index increases downward while w is positive upward.
    float adv_u = uu * du_dx + vv * du_dy - ww * du_dz;



    float adv_v = uu * dv_dx + vv * dv_dy - ww * dv_dz;



    float adv_w = uu * dw_dx + vv * dw_dy - ww * dw_dz;



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

// SSP-RK2 corrector: q^{n+1}=0.5*q^n+0.5*(q_stage+dt*L(q_stage)).
kernel void advection_ssprk2_correct(
    device const float* base_u [[buffer(0)]], device const float* base_v [[buffer(1)]], device const float* base_w [[buffer(2)]],
    device const float* stage_u [[buffer(3)]], device const float* stage_v [[buffer(4)]], device const float* stage_w [[buffer(5)]],
    device float* out_u [[buffer(6)]], device float* out_v [[buffer(7)]], device float* out_w [[buffer(8)]],
    constant GridInfo& grid [[buffer(9)]], constant float& dt [[buffer(10)]],
    uint3 id [[thread_position_in_grid]]) {
    const uint x=id.x,y=id.y,z=id.z;
    if(x>=grid.nx||y>=grid.ny||z>=grid.nz)return;
    const uint i=index3d(x,y,z,grid);
    if(x==0||y==0||z==0||x==grid.nx-1||y==grid.ny-1||z==grid.nz-1){
        out_u[i]=base_u[i];out_v[i]=base_v[i];out_w[i]=base_w[i];return;
    }
    const uint xp=index3d(x+1,y,z,grid),xm=index3d(x-1,y,z,grid);
    const uint yp=index3d(x,y+1,z,grid),ym=index3d(x,y-1,z,grid);
    const uint zp=index3d(x,y,z+1,grid),zm=index3d(x,y,z-1,grid);
    const float dudx=(stage_u[xp]-stage_u[xm])/(2.0f*grid.dx);
    const float dudy=(stage_u[yp]-stage_u[ym])/(2.0f*grid.dy);
    const float dudz=(stage_u[zp]-stage_u[zm])/(2.0f*grid.dz);
    const float dvdx=(stage_v[xp]-stage_v[xm])/(2.0f*grid.dx);
    const float dvdy=(stage_v[yp]-stage_v[ym])/(2.0f*grid.dy);
    const float dvdz=(stage_v[zp]-stage_v[zm])/(2.0f*grid.dz);
    const float dwdx=(stage_w[xp]-stage_w[xm])/(2.0f*grid.dx);
    const float dwdy=(stage_w[yp]-stage_w[ym])/(2.0f*grid.dy);
    const float dwdz=(stage_w[zp]-stage_w[zm])/(2.0f*grid.dz);
    const float su=stage_u[i],sv=stage_v[i],sw=stage_w[i];
    const float rhs_u=-(su*dudx+sv*dudy-sw*dudz);
    const float rhs_v=-(su*dvdx+sv*dvdy-sw*dvdz);
    const float rhs_w=-(su*dwdx+sv*dwdy-sw*dwdz);
    out_u[i]=0.5f*(base_u[i]+stage_u[i]+dt*rhs_u);
    out_v[i]=0.5f*(base_v[i]+stage_v[i]+dt*rhs_v);
    out_w[i]=0.5f*(base_w[i]+stage_w[i]+dt*rhs_w);
}
