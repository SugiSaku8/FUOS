#include <metal_stdlib>
using namespace metal;

struct GridInfo { uint nx; uint ny; uint nz; float dx; float dy; float dz; };
inline uint index3d(uint x, uint y, uint z, constant GridInfo& g) {
    return z * g.nx * g.ny + y * g.nx + x;
}

// Boussinesq buoyancy from EOS-80 density anomaly relative to the horizontal
// mean density at the same depth. Positive w is upward; z-index increases downward.
kernel void buoyancy_step(
    device const float* density [[buffer(0)]],
    device const float* reference_density [[buffer(1)]],
    device const float* u [[buffer(2)]],
    device const float* v [[buffer(3)]],
    device const float* w [[buffer(4)]],
    device float* out_u [[buffer(5)]],
    device float* out_v [[buffer(6)]],
    device float* out_w [[buffer(7)]],
    constant GridInfo& grid [[buffer(8)]],
    constant float& dt [[buffer(9)]],
    uint3 id [[thread_position_in_grid]]) {
    const uint x=id.x, y=id.y, z=id.z;
    if (x>=grid.nx || y>=grid.ny || z>=grid.nz) return;
    const uint i=index3d(x,y,z,grid);
    const float rho=max(density[i], 900.0f);
    const float rho_ref=max(reference_density[z], 900.0f);
    const float anomaly=(rho-rho_ref)/rho_ref;
    const float acceleration=-9.80665f*anomaly;
    out_u[i]=u[i];
    out_v[i]=v[i];
    out_w[i]=w[i]+dt*acceleration;
}
