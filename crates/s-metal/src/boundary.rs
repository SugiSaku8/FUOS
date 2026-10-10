use crate::{GpuVelocityField, MetalContext};

use s_field::{Grid4D, VelocityField};

/// Ocean velocity boundary conditions.
///
/// MVP:
/// - Prescribes velocity on the x = nx - 1 boundary face.
/// - Preserves the velocity values in the interior.
///
/// This is a test boundary condition, not a validated Kuroshio model.
pub struct BoundaryOperator;

impl BoundaryOperator {
    pub fn new(_context: &MetalContext) -> Self {
        Self
    }
    /// Construct a test inflow velocity field.
    pub fn kuroshio_inflow(
        &self,
        context: &MetalContext,
        velocity: &GpuVelocityField,
    ) -> GpuVelocityField {
        let grid_info = velocity.grid();

        let nx = grid_info.nx() as usize;
        let ny = grid_info.ny() as usize;
        let nz = grid_info.nz() as usize;

        let total = nx * ny * nz;

        let grid = Grid4D::new(
            [nx, ny, nz, 1],
            [grid_info.dx(), grid_info.dy(), grid_info.dz(), 1.0],
        );

        let mut field = VelocityField::new(grid);

        let mut u = vec![0.0_f32; total];
        let mut v = vec![0.0_f32; total];
        let w = vec![0.0_f32; total];

        if nx > 0 && ny > 0 && nz > 0 {
            for z in 0..nz {
                let depth_factor = if nz > 1 {
                    1.0 - z as f32 / (nz - 1) as f32
                } else {
                    1.0
                };

                for y in 0..ny {
                    let index = z * nx * ny + y * nx + (nx - 1);

                    u[index] = 0.8 * depth_factor;
                    v[index] = 0.15 * depth_factor;
                }
            }
        }

        field.u_mut().replace(u);
        field.v_mut().replace(v);
        field.w_mut().replace(w);

        GpuVelocityField::from_velocity(context, &field)
    }

    /// Apply inflow on one boundary face while preserving the interior.
    ///
    /// This MVP implementation transfers the field through CPU memory.
    pub fn apply_velocity(&self, context: &MetalContext, velocity: &mut GpuVelocityField) {
        let grid_info = velocity.grid();
        let nx = grid_info.nx() as usize;
        let ny = grid_info.ny() as usize;
        let nz = grid_info.nz() as usize;

        if nx == 0 || ny == 0 || nz == 0 {
            return;
        }

        // 現在の速度場をCPUへダウンロード
        let mut current = velocity.download(context);

        // 流入境界の速度場を作成
        let inflow = self.kuroshio_inflow(context, velocity);
        let inflow_cpu = inflow.download(context);

        // u, v, w をそれぞれ独立した配列として取得
        let mut u = current
            .u()
            .time_slice(0)
            .expect("Failed to access current u velocity")
            .to_vec();

        let mut v = current
            .v()
            .time_slice(0)
            .expect("Failed to access current v velocity")
            .to_vec();

        let mut w = current
            .w()
            .time_slice(0)
            .expect("Failed to access current w velocity")
            .to_vec();

        let inflow_u = inflow_cpu
            .u()
            .time_slice(0)
            .expect("Failed to access inflow u velocity");

        let inflow_v = inflow_cpu
            .v()
            .time_slice(0)
            .expect("Failed to access inflow v velocity");

        let inflow_w = inflow_cpu
            .w()
            .time_slice(0)
            .expect("Failed to access inflow w velocity");

        // x方向の端面に流入境界条件を適用
        for z in 0..nz {
            for y in 0..ny {
                let index = z * nx * ny + y * nx + (nx - 1);

                u[index] = inflow_u[index];
                v[index] = inflow_v[index];
                w[index] = inflow_w[index];
            }
        }

        // 変更した配列を速度場へ戻す
        current.u_mut().replace(u);
        current.v_mut().replace(v);
        current.w_mut().replace(w);

        // 更新した速度場をGPUへアップロード
        *velocity = GpuVelocityField::from_velocity(context, &current);
    }
}
