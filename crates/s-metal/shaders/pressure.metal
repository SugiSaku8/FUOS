#include <metal_stdlib>
using namespace metal;

struct GridInfo { uint nx; uint ny; uint nz; float dx; float dy; float dz; };
inline uint index3d(uint x, uint y, uint z, constant GridInfo& g) {
    return z * g.nx * g.ny + y * g.nx + x;
}

// Variable-density projection: div((1/rho) grad(p)) = div(u*) / dt.
// Neumann pressure boundaries are used; one cell fixes the arbitrary pressure gauge.
kernel void pressure_jacobi(
    device const float* p [[buffer(0)]],
    device const float* div [[buffer(1)]],
    device float* out [[buffer(2)]],
    constant GridInfo& grid [[buffer(3)]],
    constant float& dt [[buffer(4)]],
    device const float* rho [[buffer(5)]],
    device const float* land_mask [[buffer(6)]],
    constant uint& gauge_index [[buffer(7)]],
    uint3 id [[thread_position_in_grid]]) {
    const uint x=id.x, y=id.y, z=id.z;
    if (x>=grid.nx || y>=grid.ny || z>=grid.nz) return;
    const uint i=index3d(x,y,z,grid);
    if (land_mask[i]>0.5f || i==gauge_index || grid.dx<=0.0f || grid.dy<=0.0f || grid.dz<=0.0f || dt<=0.0f) {
        out[i]=0.0f; return;
    }
    const float inv_dx2=1.0f/(grid.dx*grid.dx);
    const float inv_dy2=1.0f/(grid.dy*grid.dy);
    const float inv_dz2=1.0f/(grid.dz*grid.dz);
    const float rhoi=max(rho[i],900.0f);
    float weighted=0.0f, sumw=0.0f;

    if (x>0) { uint j=index3d(x-1,y,z,grid); if(land_mask[j]<=0.5f){float a=2.0f/(rhoi+max(rho[j],900.0f))*inv_dx2; weighted+=a*p[j]; sumw+=a;} }
    if (x+1<grid.nx) { uint j=index3d(x+1,y,z,grid); if(land_mask[j]<=0.5f){float a=2.0f/(rhoi+max(rho[j],900.0f))*inv_dx2; weighted+=a*p[j]; sumw+=a;} }
    if (y>0) { uint j=index3d(x,y-1,z,grid); if(land_mask[j]<=0.5f){float a=2.0f/(rhoi+max(rho[j],900.0f))*inv_dy2; weighted+=a*p[j]; sumw+=a;} }
    if (y+1<grid.ny) { uint j=index3d(x,y+1,z,grid); if(land_mask[j]<=0.5f){float a=2.0f/(rhoi+max(rho[j],900.0f))*inv_dy2; weighted+=a*p[j]; sumw+=a;} }
    if (z>0) { uint j=index3d(x,y,z-1,grid); if(land_mask[j]<=0.5f){float a=2.0f/(rhoi+max(rho[j],900.0f))*inv_dz2; weighted+=a*p[j]; sumw+=a;} }
    if (z+1<grid.nz) { uint j=index3d(x,y,z+1,grid); if(land_mask[j]<=0.5f){float a=2.0f/(rhoi+max(rho[j],900.0f))*inv_dz2; weighted+=a*p[j]; sumw+=a;} }

    out[i] = sumw > 0.0f ? (weighted - div[i]/dt)/sumw : 0.0f;
}
