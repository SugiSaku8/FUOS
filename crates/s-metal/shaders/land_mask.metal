#include <metal_stdlib>
using namespace metal;

struct GridInfo
{
    uint nx;
    uint ny;
    uint nz;
};

kernel void land_mask_step(

    device const float *mask [[buffer(0)]],

    device const float *in_u [[buffer(1)]],
    device const float *in_v [[buffer(2)]],
    device const float *in_w [[buffer(3)]],

    device float *out_u [[buffer(4)]],
    device float *out_v [[buffer(5)]],
    device float *out_w [[buffer(6)]],

    constant GridInfo &grid [[buffer(7)]],

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

    if (mask[i] > 0.5f)
    {
        out_u[i] = 0.0f;
        out_v[i] = 0.0f;
        out_w[i] = 0.0f;
    }
    else
    {
        out_u[i] = in_u[i];
        out_v[i] = in_v[i];
        // Rigid-lid / impermeable-bottom approximation: the current model has
        // no prognostic free-surface elevation, so normal velocity is constrained.
        bool vertical_wall = (gid.z == 0 || gid.z + 1 >= grid.nz);
        if (!vertical_wall && gid.z + 1 < grid.nz) {
            uint below = (gid.z + 1) * grid.nx * grid.ny + gid.y * grid.nx + gid.x;
            vertical_wall = mask[below] > 0.5f;
        }
        out_w[i] = vertical_wall ? 0.0f : in_w[i];
    }
}
