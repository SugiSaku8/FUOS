#include <metal_stdlib>
using namespace metal;
struct GridInfo { uint nx; uint ny; uint nz; float dx; float dy; float dz; };
inline uint index3d(uint x,uint y,uint z,constant GridInfo& g){return z*g.nx*g.ny+y*g.nx+x;}

kernel void gradient_pressure(
    device const float* pressure [[buffer(0)]],
    device const float* u [[buffer(1)]], device const float* v [[buffer(2)]], device const float* w [[buffer(3)]],
    device float* u_out [[buffer(4)]], device float* v_out [[buffer(5)]], device float* w_out [[buffer(6)]],
    constant GridInfo& grid [[buffer(7)]], constant float& dt [[buffer(8)]],
    device const float* density [[buffer(9)]], device const float* land_mask [[buffer(10)]], uint3 gid [[thread_position_in_grid]]) {
    const uint x=gid.x,y=gid.y,z=gid.z;
    if(x>=grid.nx||y>=grid.ny||z>=grid.nz)return;
    const uint i=index3d(x,y,z,grid);
    if(land_mask[i]>0.5f){u_out[i]=0.0f;v_out[i]=0.0f;w_out[i]=0.0f;return;}
    const float rhoi=max(density[i],900.0f);
    u_out[i]=u[i]; v_out[i]=v[i]; w_out[i]=w[i];
    if(x+1<grid.nx){uint j=index3d(x+1,y,z,grid);if(land_mask[j]<=0.5f){float inv_rho_face=2.0f/(rhoi+max(density[j],900.0f));u_out[i]=u[i]-dt*inv_rho_face*(pressure[j]-pressure[i])/grid.dx;}}
    if(y+1<grid.ny){uint j=index3d(x,y+1,z,grid);if(land_mask[j]<=0.5f){float inv_rho_face=2.0f/(rhoi+max(density[j],900.0f));v_out[i]=v[i]-dt*inv_rho_face*(pressure[j]-pressure[i])/grid.dy;}}
    // Depth index grows downward, while w is positive upward: sign is reversed.
    if(z+1<grid.nz){uint j=index3d(x,y,z+1,grid);if(land_mask[j]<=0.5f){float inv_rho_face=2.0f/(rhoi+max(density[j],900.0f));w_out[i]=w[i]+dt*inv_rho_face*(pressure[j]-pressure[i])/grid.dz;}}
}
