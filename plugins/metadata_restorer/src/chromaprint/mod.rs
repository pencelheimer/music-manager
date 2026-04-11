mod models;

pub use models::Fingerprint;

use std::path::Path;
use tokio::process::Command;
use tracing::instrument;

use crate::error::FpCalcError;

/// Invokes `fpcalc` for the specified file.
#[instrument(skip_all, fields( path = %path.as_ref().display()))]
pub async fn calculate_fingerprint(path: impl AsRef<Path>) -> Result<Fingerprint, FpCalcError> {
    let output = Command::new("fpcalc")
        .arg("-json")
        .arg(path.as_ref())
        .output()
        .await?;

    if !output.status.success() {
        return Err(FpCalcError::process_failed(output.status));
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}
