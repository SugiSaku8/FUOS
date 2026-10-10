#include <metal_stdlib>
using namespace metal;

struct GridInfo {
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
) {
    return z * grid.nx * grid.ny
         + y * grid.nx
         + x;
}

kernel void divergence_step(
    device const float* u       [[buffer(0)]],
    device const float* v       [[buffer(1)]],
    device const float* w       [[buffer(2)]],
    device float* divergence    [[buffer(3)]],
    constant GridInfo& grid     [[buffer(4)]],
    uint3 gid                   [[thread_position_in_grid]]
) {
    const uint x = gid.x;
    const uint y = gid.y;
    const uint z = gid.z;

    if (
        x >= grid.nx ||
        y >= grid.ny ||
        z >= grid.nz
    ) {
        return;
    }

    const uint i = index3d(x, y, z, grid);

    // 境界セルでは発散を評価しない。
    if (
        x == 0 || x == grid.nx - 1 ||
        y == 0 || y == grid.ny - 1 ||
        z == 0 || z == grid.nz - 1
    ) {
        divergence[i] = 0.0f;
        return;
    }

    const uint ixm = index3d(x - 1, y, z, grid);
    const uint iym = index3d(x, y - 1, z, grid);
    const uint izm = index3d(x, y, z - 1, grid);

    // 後退差分。
    const float du_dx =
        (u[i] - u[ixm]) / grid.dx;

    const float dv_dy =
        (v[i] - v[iym]) / grid.dy;

    const float dw_dz =
        (w[i] - w[izm]) / grid.dz;

    divergence[i] = du_dx + dv_dy + dw_dz;
}
