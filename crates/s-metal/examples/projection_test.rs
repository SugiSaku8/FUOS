use s_metal::{
    DivergenceOperator, GpuVelocityField, GradientPressureOperator, GridInfo, MetalContext,
    PressureSolver,
};

use std::f64::consts::PI;
use std::time::Instant;

const JACOBI_ITERATIONS: usize = 1000;

fn index3d(x: usize, y: usize, z: usize, nx: usize, ny: usize) -> usize {
    z * nx * ny + y * nx + x
}

/// 内部セルの発散 RMS を計算する。
fn rms_interior(values: &[f32], nx: usize, ny: usize, nz: usize) -> f64 {
    if nx < 3 || ny < 3 || nz < 3 {
        return 0.0;
    }

    let mut sum_squared = 0.0_f64;
    let mut count = 0_usize;
    for z in 1..nz - 1 {
        for y in 1..ny - 1 {
            for x in 1..nx - 1 {
                let i = index3d(x, y, z, nx, ny);
                let value = values[i] as f64;
                sum_squared += value * value;
                count += 1;
            }
        }
    }
    if count == 0 {
        return 0.0;
    }
    (sum_squared / count as f64).sqrt()
}

/// 圧力方程式の相対残差を計算する。
///
/// 対象方程式:
///     laplacian(p) = (rho / dt) * divergence
///
/// 戻り値:
///     (残差 RMS, 相対残差)
fn pressure_residual_rms(
    pressure: &[f32],
    divergence: &[f32],
    nx: usize,
    ny: usize,
    nz: usize,
    dx: f32,
    dy: f32,
    dz: f32,
    dt: f32,
    rho: f32,
) -> (f64, f64) {
    if nx < 3 || ny < 3 || nz < 3 {
        return (0.0, 0.0);
    }

    let dx = dx as f64;
    let dy = dy as f64;
    let dz = dz as f64;
    let wx = 1.0 / (dx * dx);
    let wy = 1.0 / (dy * dy);
    let wz = 1.0 / (dz * dz);
    let scale = rho as f64 / dt as f64;
    let mut residual_squared_sum = 0.0_f64;
    let mut rhs_squared_sum = 0.0_f64;
    let mut count = 0_usize;
    for z in 1..nz - 1 {
        for y in 1..ny - 1 {
            for x in 1..nx - 1 {
                let i = index3d(x, y, z, nx, ny);
                let xm = index3d(x - 1, y, z, nx, ny);
                let xp = index3d(x + 1, y, z, nx, ny);
                let ym = index3d(x, y - 1, z, nx, ny);
                let yp = index3d(x, y + 1, z, nx, ny);
                let zm = index3d(x, y, z - 1, nx, ny);
                let zp = index3d(x, y, z + 1, nx, ny);
                let p = pressure[i] as f64;
                let laplacian = wx * (pressure[xm] as f64 - 2.0 * p + pressure[xp] as f64)
                    + wy * (pressure[ym] as f64 - 2.0 * p + pressure[yp] as f64)
                    + wz * (pressure[zm] as f64 - 2.0 * p + pressure[zp] as f64);
                let rhs = scale * divergence[i] as f64;
                let residual = laplacian - rhs;
                residual_squared_sum += residual * residual;
                rhs_squared_sum += rhs * rhs;
                count += 1;
            }
        }
    }
    if count == 0 {
        return (0.0, 0.0);
    }
    let residual_rms = (residual_squared_sum / count as f64).sqrt();
    let rhs_rms = (rhs_squared_sum / count as f64).sqrt();
    let relative_residual = if rhs_rms > 1.0e-30 {
        residual_rms / rhs_rms
    } else {
        0.0
    };
    (residual_rms, relative_residual)
}

/// 空間的に変化する速度場を作る。
fn make_test_velocity(
    context: &MetalContext,
    grid: GridInfo,
    nx: usize,
    ny: usize,
    nz: usize,
) -> GpuVelocityField {
    let mut u = vec![0.0_f32; grid.count()];
    let mut v = vec![0.0_f32; grid.count()];
    let mut w = vec![0.0_f32; grid.count()];

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let i = index3d(x, y, z, nx, ny);
                let xf = x as f64 / (nx - 1) as f64;
                let yf = y as f64 / (ny - 1) as f64;
                let zf = z as f64 / (nz - 1) as f64;
                u[i] = (2.0 * PI * xf).sin() as f32;
                v[i] = (2.0 * PI * yf).sin() as f32;
                w[i] = (2.0 * PI * zf).sin() as f32;
            }
        }
    }
    let velocity = GpuVelocityField::empty(context, grid);
    velocity.u.upload(context, &u);
    velocity.v.upload(context, &v);
    velocity.w.upload(context, &w);
    velocity
}

/// 一様速度場を作る。
fn make_uniform_velocity(context: &MetalContext, grid: GridInfo) -> GpuVelocityField {
    let count = grid.count();

    let velocity = GpuVelocityField::empty(context, grid);
    velocity.u.upload(context, &vec![1.25; count]);
    velocity.v.upload(context, &vec![-0.5; count]);
    velocity.w.upload(context, &vec![0.75; count]);
    velocity
}

struct ProjectionResult {
    iterations: usize,
    divergence_before: f64,
    divergence_after: f64,
    reduction_percent: f64,
    residual_rms: f64,
    relative_residual: f64,
    elapsed_ms: f64,
}

/// 速度場に圧力補正を適用し、結果を測定する。
fn run_projection_case(
    context: &MetalContext,

    divergence_operator: &DivergenceOperator,

    pressure_solver: &PressureSolver,

    gradient_operator: &GradientPressureOperator,

    nx: usize,

    ny: usize,

    nz: usize,

    dt: f32,

    rho: f32,

    iterations: usize,
) -> ProjectionResult {
    let dx = 1.0_f32;
    let dy = 1.0_f32;
    let dz = 1.0_f32;

    let grid = GridInfo::new(nx as u32, ny as u32, nz as u32, dx, dy, dz);
    let velocity = make_test_velocity(context, grid, nx, ny, nz);
    let start = Instant::now();
    // 1. 補正前の発散
    let divergence_before_field = divergence_operator.compute(context, &velocity);
    let divergence_before_values = divergence_before_field.download(context);
    let divergence_before = rms_interior(&divergence_before_values, nx, ny, nz);
    // 2. 圧力ポアソン方程式を解く
    let pressure = pressure_solver.solve(context, &divergence_before_field, dt, rho, iterations);
    let pressure_values = pressure.download(context);
    // 3. 圧力方程式の残差
    let (residual_rms, relative_residual) = pressure_residual_rms(
        &pressure_values,
        &divergence_before_values,
        nx,
        ny,
        nz,
        dx,
        dy,
        dz,
        dt,
        rho,
    );
    // 4. 圧力勾配による速度補正
    let corrected_velocity = gradient_operator.compute(context, &velocity, &pressure, dt, rho);
    // 5. 補正後の発散
    let divergence_after_field = divergence_operator.compute(context, &corrected_velocity);
    let divergence_after_values = divergence_after_field.download(context);
    let divergence_after = rms_interior(&divergence_after_values, nx, ny, nz);
    let reduction_percent = if divergence_before > 1.0e-30 {
        (1.0 - divergence_after / divergence_before) * 100.0
    } else {
        0.0
    };
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    ProjectionResult {
        iterations,
        divergence_before,
        divergence_after,
        reduction_percent,
        residual_rms,
        relative_residual,
        elapsed_ms,
    }
}

fn print_result(label: &str, nx: usize, dt: f32, result: &ProjectionResult) {
    println!();
    println!("— {label} —");
    println!("Grid: {} x {} x {}", nx, nx, nx);
    println!("dt: {:.4}", dt);
    println!("Jacobi iterations: {}", result.iterations);
    println!("Divergence RMS before: {:.8e}", result.divergence_before);
    println!("Divergence RMS after:  {:.8e}", result.divergence_after);
    println!("Reduction:             {:.4}%", result.reduction_percent);
    println!("Pressure residual RMS: {:.8e}", result.residual_rms);
    println!("Relative residual:     {:.8e}", result.relative_residual);
    println!("Elapsed:               {:.2} ms", result.elapsed_ms);
}

fn main() {
    let context = MetalContext::new();

    let divergence_operator = DivergenceOperator::new(&context);
    let pressure_solver = PressureSolver::new(&context);
    let gradient_operator = GradientPressureOperator::new(&context);
    let rho = 1025.0_f32;
    println!("======================================");
    println!(" Suruga-Wan Pressure Projection Tests");
    println!("======================================");
    // Test A: 一様速度場
    {
        let n = 24;
        let grid = GridInfo::new(n as u32, n as u32, n as u32, 1.0, 1.0, 1.0);
        let velocity = make_uniform_velocity(&context, grid);
        let divergence_field = divergence_operator.compute(&context, &velocity);
        let divergence_values = divergence_field.download(&context);
        let rms = rms_interior(&divergence_values, n, n, n);
        println!();
        println!("--- Test A: Uniform Velocity ---");
        println!("Grid: {n} x {n} x {n}");
        println!("Divergence RMS: {:.8e}", rms);
        if rms < 1.0e-6 {
            println!("Result: PASS");
        } else {
            println!("Result: FAIL");
        }
    }
    // Test B: 基準ケース
    // Test B: Jacobi 法の収束性テスト
    let grid_sizes = [16_usize, 24, 32];
    let iteration_counts = [100_usize, 500, 1000, 2000, 5000];
    let dt = 0.01_f32;

    for n in grid_sizes {
        for iterations in iteration_counts {
            let result = run_projection_case(
                &context,
                &divergence_operator,
                &pressure_solver,
                &gradient_operator,
                n,
                n,
                n,
                dt,
                rho,
                iterations,
            );

            print_result("Jacobi Convergence", n, dt, &result);
        }
    }

    println!();
    println!("======================================");
    println!(" All projection cases completed.");
    println!("======================================");
}
