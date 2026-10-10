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

kernel void gradient_pressure(
    device const float* pressure [[buffer(0)]],

    device const float* u         [[buffer(1)]],
    device const float* v         [[buffer(2)]],
    device const float* w         [[buffer(3)]],

    device float* u_out           [[buffer(4)]],
    device float* v_out           [[buffer(5)]],
    device float* w_out           [[buffer(6)]],

    constant GridInfo& grid       [[buffer(7)]],

    constant float& dt             [[buffer(8)]],
    constant float& rho            [[buffer(9)]],

    uint3 gid                      [[thread_position_in_grid]]
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

    const float factor = dt / rho;

    // 各方向の前進差分を個別に計算する。
    //
    // 対応する方向に隣接セルが存在しない場合は、
    // その速度成分を変更しない。

    // X direction
    if (x + 1 < grid.nx) {
        const uint ixp = index3d(x + 1, y, z, grid);

        const float dp_dx =
            (pressure[ixp] - pressure[i]) / grid.dx;

        u_out[i] = u[i] - factor * dp_dx;
    } else {
        u_out[i] = u[i];
    }

    // Y direction
    if (y + 1 < grid.ny) {
        const uint iyp = index3d(x, y + 1, z, grid);

        const float dp_dy =
            (pressure[iyp] - pressure[i]) / grid.dy;

        v_out[i] = v[i] - factor * dp_dy;
    } else {
        v_out[i] = v[i];
    }

    // Z direction
    if (z + 1 < grid.nz) {
        const uint izp = index3d(x, y, z + 1, grid);

        const float dp_dz =
            (pressure[izp] - pressure[i]) / grid.dz;

        w_out[i] = w[i] - factor * dp_dz;
    } else {
        w_out[i] = w[i];
    }
}
