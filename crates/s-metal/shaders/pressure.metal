#include <metal_stdlib>

using namespace metal;

struct GridInfo
{
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
)
{
    return z * grid.nx * grid.ny
         + y * grid.nx
         + x;
}

kernel void pressure_jacobi(
    device const float* p [[buffer(0)]],
    device const float* div [[buffer(1)]],
    device float* out [[buffer(2)]],
    constant GridInfo& grid [[buffer(3)]],

    constant float& dt [[buffer(4)]],
    constant float& rho [[buffer(5)]],

    uint3 id [[thread_position_in_grid]]
)
{
    const uint x = id.x;
    const uint y = id.y;
    const uint z = id.z;

    if (
        x >= grid.nx ||
        y >= grid.ny ||
        z >= grid.nz
    )
    {
        return;
    }

    const uint i = index3d(x, y, z, grid);

    // Dirichlet boundary condition: p = 0
    if (
        x == 0 || y == 0 || z == 0 ||
        x == grid.nx - 1 ||
        y == grid.ny - 1 ||
        z == grid.nz - 1
    )
    {
        out[i] = 0.0;
        return;
    }

    // Prevent division by invalid physical parameters.
    if (
        grid.dx <= 0.0 ||
        grid.dy <= 0.0 ||
        grid.dz <= 0.0 ||
        dt <= 0.0 ||
        rho <= 0.0
    )
    {
        out[i] = 0.0;
        return;
    }

    const uint xm = index3d(x - 1, y, z, grid);
    const uint xp = index3d(x + 1, y, z, grid);

    const uint ym = index3d(x, y - 1, z, grid);
    const uint yp = index3d(x, y + 1, z, grid);

    const uint zm = index3d(x, y, z - 1, grid);
    const uint zp = index3d(x, y, z + 1, grid);

    // Inverse squared grid spacing.
    const float wx = 1.0 / (grid.dx * grid.dx);
    const float wy = 1.0 / (grid.dy * grid.dy);
    const float wz = 1.0 / (grid.dz * grid.dz);

    // Weighted sum of neighboring pressure values.
    const float neighbors =
          wx * (p[xm] + p[xp])
        + wy * (p[ym] + p[yp])
        + wz * (p[zm] + p[zp]);

    // Physical pressure Poisson equation:
    // Laplacian(p) = (rho / dt) * divergence(u*)
    const float rhs = (rho / dt) * div[i];

    // Jacobi iteration for anisotropic grid spacing.
    out[i] =
        (neighbors - rhs)
        / (2.0 * (wx + wy + wz));
}
