//! UNESCO 1983 (EOS-80) seawater density approximation.
//!
//! Temperature is ITS-90 degrees Celsius, salinity is practical salinity (PSU),
//! depth is positive downward in metres. This is not TEOS-10; it is a documented,
//! self-contained approximation suitable for the current lightweight prototype.

/// Density in kg m^-3, including a first-order pressure/compressibility correction.
/// Returns `None` for invalid or out-of-domain input rather than propagating NaNs.
pub fn seawater_density_eos80(temperature_c: f32, salinity_psu: f32, depth_m: f32) -> Option<f32> {
    if !temperature_c.is_finite() || !salinity_psu.is_finite() || !depth_m.is_finite()
        || !(-3.0..=45.0).contains(&temperature_c)
        || !(0.0..=45.0).contains(&salinity_psu)
        || depth_m < 0.0
    {
        return None;
    }

    let t = temperature_c as f64;
    let s = salinity_psu as f64;
    let sqrt_s = s.sqrt();

    // Density at atmospheric pressure, UNESCO 1983 polynomial.
    let rho_w = 999.842_594
        + 6.793_952e-2 * t
        - 9.095_290e-3 * t.powi(2)
        + 1.001_685e-4 * t.powi(3)
        - 1.120_083e-6 * t.powi(4)
        + 6.536_332e-9 * t.powi(5);
    let a = 0.824_493 - 4.089_9e-3 * t + 7.643_8e-5 * t.powi(2)
        - 8.246_7e-7 * t.powi(3) + 5.387_5e-9 * t.powi(4);
    let b = -5.724_66e-3 + 1.022_7e-4 * t - 1.654_6e-6 * t.powi(2);
    let rho0 = rho_w + a * s + b * s * sqrt_s + 4.831_4e-4 * s * s;

    // Secant bulk modulus (bar), including pressure terms. Pressure is approximated
    // hydrostatically as depth/10.1325 dbar, then converted to bar.
    let p = depth_m as f64 / 10.1325;
    let kw = 19_652.21 + 148.4206 * t - 2.327105 * t.powi(2)
        + 1.360477e-2 * t.powi(3) - 5.155288e-5 * t.powi(4);
    let aa = 54.6746 - 0.603459 * t + 1.09987e-2 * t.powi(2) - 6.1670e-5 * t.powi(3);
    let bb = 7.944e-2 + 1.6483e-2 * t - 5.3009e-4 * t.powi(2);
    let k0 = kw + aa * s + bb * s * sqrt_s;
    let k1 = 3.239908 + 1.43713e-3 * t + 1.16092e-4 * t.powi(2)
        - 5.77905e-7 * t.powi(3)
        + (2.2838e-3 - 1.0981e-5 * t - 1.6078e-6 * t.powi(2)) * s
        + 1.91075e-4 * s * sqrt_s;
    let k2 = 8.50935e-5 - 6.12293e-6 * t + 5.2787e-8 * t.powi(2)
        + (-9.9348e-7 + 2.0816e-8 * t + 9.1697e-10 * t.powi(2)) * s;
    let bulk_modulus = k0 + k1 * p + k2 * p * p;
    let rho = rho0 / (1.0 - p / bulk_modulus);

    if rho.is_finite() && (900.0..=1_200.0).contains(&rho) {
        Some(rho as f32)
    } else {
        None
    }
}

/// Build a density field and a horizontal reference density for each depth layer.
/// Cells marked by the dataset builder as land/sub-bottom are represented by T=S=0;
/// they are excluded from layer means and assigned that layer's mean density afterward.
pub fn density_profile(
    temperature: &[f32], salinity: &[f32], nx: usize, ny: usize, nz: usize, dz: f32,
) -> (Vec<f32>, Vec<f32>) {
    let n = nx.saturating_mul(ny).saturating_mul(nz);
    if temperature.len() != n || salinity.len() != n || nx == 0 || ny == 0 || nz == 0 {
        return (vec![1025.0; n], vec![1025.0; nz]);
    }
    let plane = nx * ny;
    let mut density = vec![f32::NAN; n];
    let mut reference = vec![1025.0; nz];
    for z in 0..nz {
        let depth = z as f32 * dz.max(0.0);
        let start = z * plane;
        let end = start + plane;
        let mut sum = 0.0_f64;
        let mut count = 0usize;
        for i in start..end {
            let t = temperature[i];
            let s = salinity[i];
            // The dataset mask currently writes (T,S)=(0,0) on land and below seabed.
            if t == 0.0 && s == 0.0 { continue; }
            if let Some(rho) = seawater_density_eos80(t, s, depth) {
                density[i] = rho;
                sum += rho as f64;
                count += 1;
            }
        }
        if count > 0 {
            reference[z] = (sum / count as f64) as f32;
        }
        for i in start..end {
            if !density[i].is_finite() { density[i] = reference[z]; }
        }
    }
    (density, reference)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_behaves_physically_over_typical_ocean_ranges() {
        let cool = seawater_density_eos80(5.0, 35.0, 0.0).unwrap();
        let warm = seawater_density_eos80(20.0, 35.0, 0.0).unwrap();
        let salty = seawater_density_eos80(10.0, 36.0, 0.0).unwrap();
        let fresh = seawater_density_eos80(10.0, 34.0, 0.0).unwrap();
        let deep = seawater_density_eos80(10.0, 35.0, 4000.0).unwrap();
        assert!(cool > warm);
        assert!(salty > fresh);
        assert!(deep > seawater_density_eos80(10.0, 35.0, 0.0).unwrap());
    }

    #[test]
    fn density_profile_excludes_zero_zero_masked_cells_from_reference() {
        let t = vec![15.0, 0.0, 15.0, 0.0];
        let s = vec![35.0, 0.0, 35.0, 0.0];
        let (rho, reference) = density_profile(&t, &s, 2, 2, 1, 10.0);
        assert!((rho[0] - reference[0]).abs() < 1.0e-4);
        assert!((rho[1] - reference[0]).abs() < 1.0e-4);
        assert!(reference[0] > 1000.0 && reference[0] < 1040.0);
    }
}
