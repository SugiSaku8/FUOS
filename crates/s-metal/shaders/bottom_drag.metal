#include <metal_stdlib>
using namespace metal;
struct GridInfo { uint nx; uint ny; uint nz; float dx; float dy; float dz; };
inline uint index3d(uint x,uint y,uint z,constant GridInfo& g){return z*g.nx*g.ny+y*g.nx+x;}

// Quadratic bottom stress on the deepest wet cell: a = -Cd |U| U / layer_thickness.
kernel void bottom_drag_step(
    device const float* u [[buffer(0)]],device const float* v [[buffer(1)]],device const float* w [[buffer(2)]],
    device float* out_u [[buffer(3)]],device float* out_v [[buffer(4)]],device float* out_w [[buffer(5)]],
    device const float* land_mask [[buffer(6)]],constant GridInfo& grid [[buffer(7)]],
    constant float& dt [[buffer(8)]],constant float& drag_coefficient [[buffer(9)]],
    uint3 gid [[thread_position_in_grid]]){
    const uint x=gid.x,y=gid.y,z=gid.z;
    if(x>=grid.nx||y>=grid.ny||z>=grid.nz)return;
    const uint i=index3d(x,y,z,grid);
    if(land_mask[i]>0.5f){out_u[i]=0.0f;out_v[i]=0.0f;out_w[i]=0.0f;return;}
    float uu=u[i],vv=v[i];
    bool bottom=(z+1>=grid.nz);
    if(!bottom){const uint below=index3d(x,y,z+1,grid);bottom=land_mask[below]>0.5f;}
    if(bottom){
        const float speed=sqrt(uu*uu+vv*vv);
        const float factor=drag_coefficient*speed*dt/max(grid.dz,1.0f);
        const float scale=1.0f/(1.0f+factor);
        uu*=scale;vv*=scale;
    }
    out_u[i]=uu;out_v[i]=vv;out_w[i]=w[i];
}
