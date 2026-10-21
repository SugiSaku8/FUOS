#include <metal_stdlib>
using namespace metal;
struct GridInfo { uint nx; uint ny; uint nz; float dx; float dy; float dz; };
inline uint index3d(uint x,uint y,uint z,constant GridInfo& g){return z*g.nx*g.ny+y*g.nx+x;}

// mode=1: prescribe the source-data values captured from the initial eastern face.
// mode=0: legacy synthetic test inflow (only for standalone solver examples).
kernel void boundary_step(
    device const float* in_u [[buffer(0)]], device const float* in_v [[buffer(1)]], device const float* in_w [[buffer(2)]],
    device float* out_u [[buffer(3)]], device float* out_v [[buffer(4)]], device float* out_w [[buffer(5)]],
    constant GridInfo& grid [[buffer(6)]],
    device const float* face_u [[buffer(7)]], device const float* face_v [[buffer(8)]], device const float* face_w [[buffer(9)]],
    constant uint& mode [[buffer(10)]], uint3 gid [[thread_position_in_grid]]) {
    const uint x=gid.x,y=gid.y,z=gid.z;
    if(x>=grid.nx||y>=grid.ny||z>=grid.nz)return;
    const uint i=index3d(x,y,z,grid);
    out_u[i]=in_u[i]; out_v[i]=in_v[i]; out_w[i]=in_w[i];
    if(x==grid.nx-1){
        if(mode==1){const uint face=z*grid.ny+y;out_u[i]=face_u[face];out_v[i]=face_v[face];out_w[i]=face_w[face];}
        else {const float depth_factor=grid.nz>1 ? 1.0f-float(z)/float(grid.nz-1) : 1.0f;out_u[i]=0.8f*depth_factor;out_v[i]=0.15f*depth_factor;out_w[i]=0.0f;}
    }
}
