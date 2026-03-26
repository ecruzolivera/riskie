use crate::t;
use anyhow::Result;
use std::process::Command;

/// Prompt user for password using available method
/// Tries: systemd-ask-password -> zenity -> fails
/// Returns Ok(Some(password)) if user entered password
/// Returns Ok(None) if user cancelled or timed out
#[allow(dead_code)]
pub fn prompt_password(device_label: &str) -> Result<Option<String>> {
    let message = t!("Enter passphrase for {}", device_label);

    // First try systemd-ask-password (works with password agents)
    tracing::info!("Attempting password prompt with systemd-ask-password");
    let output = Command::new("systemd-ask-password")
        .arg("--icon=drive-removable-media-usb")
        .arg("--keyname=riskie")
        .arg("--accept-cached")
        .arg("--timeout=1")
        .arg(&message)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                tracing::info!("Password prompt successful via systemd-ask-password");
                let password = String::from_utf8_lossy(&output.stdout)
                    .trim_end_matches('\n')
                    .to_string();
                return Ok(Some(password));
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("systemd-ask-password failed: {}", stderr);
            }
        }
        Err(e) => {
            tracing::warn!("systemd-ask-password not available: {}", e);
        }
    }

    // Fallback to zenity (common GUI dialog)
    tracing::info!("Attempting password prompt with zenity");
    let output = Command::new("zenity")
        .arg("--entry")
        .arg("--hide-text")
        .arg("--icon=drive-removable-media-usb")
        .arg(format!("--text={}", message))
        .arg(format!("--title={}", device_label))
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                tracing::info!("Password prompt successful via zenity");
                let password = String::from_utf8_lossy(&output.stdout)
                    .trim_end_matches('\n')
                    .to_string();
                Ok(Some(password))
            } else {
                tracing::info!("Password prompt cancelled via zenity");
                Ok(None)
            }
        }
        Err(e) => {
            tracing::error!(
                "Failed to prompt password (no systemd-ask-password or zenity): {}",
                e
            );
            Ok(None)
        }
    }
}
