#include <metal_stdlib>
using namespace metal;
struct GridInfo { uint nx; uint ny; uint nz; float dx; float dy; float dz; };
inline uint index3d(uint x,uint y,uint z,constant GridInfo& g){return z*g.nx*g.ny+y*g.nx+x;}

// Exact rotation for du/dt=f*v, dv/dt=-f*u. This preserves horizontal speed
// and is unconditionally stable for the Coriolis-only substep.
kernel void coriolis_step(
    device const float* u [[buffer(0)]], device const float* v [[buffer(1)]], device const float* w [[buffer(2)]],
    device float* out_u [[buffer(3)]], device float* out_v [[buffer(4)]], device float* out_w [[buffer(5)]],
    device const float* f_by_latitude [[buffer(6)]], constant GridInfo& grid [[buffer(7)]],
    constant float& dt [[buffer(8)]], uint3 gid [[thread_position_in_grid]]) {
    if(gid.x>=grid.nx||gid.y>=grid.ny||gid.z>=grid.nz)return;
    const uint i=index3d(gid.x,gid.y,gid.z,grid);
    const float theta=f_by_latitude[gid.y]*dt;
    const float c=cos(theta), s=sin(theta);
    out_u[i]=c*u[i]+s*v[i];
    out_v[i]=c*v[i]-s*u[i];
    out_w[i]=w[i];
}
