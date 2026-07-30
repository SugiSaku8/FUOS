#include <metal_stdlib>
using namespace metal;

struct GridInfo {
    uint nx;
    uint ny;
    uint nz;
};

kernel void boundary_step(

    device const float* in_u [[buffer(0)]],
    device const float* in_v [[buffer(1)]],
    device const float* in_w [[buffer(2)]],

    device float* out_u [[buffer(3)]],
    device float* out_v [[buffer(4)]],
    device float* out_w [[buffer(5)]],

    constant GridInfo& grid [[buffer(6)]],

    constant float& inflow_u [[buffer(7)]],
    constant float& inflow_v [[buffer(8)]],
    constant float& inflow_w [[buffer(9)]],

    uint3 gid [[thread_position_in_grid]]

)
{
    if (gid.x >= grid.nx ||
        gid.y >= grid.ny ||
        gid.z >= grid.nz)
        return;

    uint i =
        gid.z * grid.nx * grid.ny +
        gid.y * grid.nx +
        gid.x;

    float u = in_u[i];
    float v = in_v[i];
    float w = in_w[i];

    //
    // West boundary inflow
    //
    if (gid.x == 0)
    {
        u = inflow_u;
        v = inflow_v;
        w = inflow_w;
    }

    out_u[i] = u;
    out_v[i] = v;
    out_w[i] = w;
}
