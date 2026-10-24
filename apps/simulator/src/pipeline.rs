//! Single orchestration layer for diagnosing sources, building derived fields,
//! generating SOF products, and preparing the data consumed by the Simulator.
use anyhow::{Context, Result, ensure};

use crate::{config::Config, dataset::{self, DatasetBundle}, diagnostics::{self, DiagnosticReport}};

/// Run the shared source diagnostic, print it, and persist a machine-readable report.
pub fn diagnose_inputs(config: &Config) -> Result<DiagnosticReport> {
    let report = diagnostics::diagnose(config);
    diagnostics::print_report(&report);
    let report_path = config.output_root().join("diagnostics.json");
    diagnostics::write_json(&report, &report_path)
        .with_context(|| format!("診断レポートを保存できません: {}", report_path.display()))?;
    println!("診断JSONを書き込みました: {}", report_path.display());
    Ok(report)
}

/// Shared diagnostic-gated builder for creating the SOF consumed by Simulator.
pub fn build(config: &Config) -> Result<DatasetBundle> {
    let report = diagnose_inputs(config)?;
    build_with_report(config, &report)
}

/// Continue from an already displayed diagnostic (used by the Y/N wizard to avoid
/// running the same preflight twice). Warnings are allowed; errors are not.
pub fn build_with_report(config: &Config, report: &DiagnosticReport) -> Result<DatasetBundle> {
    ensure!(!report.has_errors(), "元データ診断に ERROR があります。cargo run -p simulator -- diagnose で詳細を確認してください。");
    dataset::build_from_validated(config)
}
