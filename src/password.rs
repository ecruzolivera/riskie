use crate::t;
use anyhow::Result;
use std::process::Command;

/// Prompt user for password using systemd-ask-password
/// Returns Ok(Some(password)) if user entered password
/// Returns Ok(None) if user cancelled or timed out
#[allow(dead_code)]
pub fn prompt_password(device_label: &str) -> Result<Option<String>> {
    let message = t!("Enter passphrase for {}", device_label);

    let output = Command::new("systemd-ask-password")
        .arg("--icon=drive-removable-media-usb")
        .arg("--keyname=riskie")
        .arg("--accept-cached")
        .arg("--timeout=120")
        .arg(&message)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let password = String::from_utf8_lossy(&output.stdout)
                    .trim_end_matches('\n')
                    .to_string();
                Ok(Some(password))
            } else {
                // User cancelled or timeout
                Ok(None)
            }
        }
        Err(e) => {
            tracing::warn!("Failed to prompt password: {}", e);
            Ok(None)
        }
    }
}
