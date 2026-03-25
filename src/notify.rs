use crate::t;
use notify_rust::{Notification, Urgency};

pub async fn notify_mount_success(device_label: String, mount_point: String) {
    let device_label = device_label.clone();
    let mount_point = mount_point.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = Notification::new()
            .summary(&t!("Device Mounted"))
            .body(&t!("{}\nMounted at {}", device_label, mount_point))
            .icon("drive-removable-media")
            .urgency(Urgency::Normal)
            .timeout(3000)
            .show()
        {
            tracing::error!("Failed to show notification: {}", e);
        }
    })
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to spawn notification task: {}", e));
}

pub async fn notify_mount_error(device_label: String, error: String) {
    tokio::task::spawn_blocking(move || {
        if let Err(e) = Notification::new()
            .summary(&t!("Mount Failed"))
            .body(&format!("{}: {}", device_label, error))
            .icon("dialog-error")
            .urgency(Urgency::Critical)
            .timeout(5000)
            .show()
        {
            tracing::error!("Failed to show notification: {}", e);
        }
    })
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to spawn notification task: {}", e));
}

pub async fn notify_unmount_success(device_label: String) {
    tokio::task::spawn_blocking(move || {
        if let Err(e) = Notification::new()
            .summary(&t!("Device Unmounted"))
            .body(&t!("{} safely removed", device_label))
            .icon("drive-removable-media")
            .urgency(Urgency::Normal)
            .timeout(3000)
            .show()
        {
            tracing::error!("Failed to show notification: {}", e);
        }
    })
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to spawn notification task: {}", e));
}

pub async fn notify_unmount_error(device_label: String, error: String) {
    tokio::task::spawn_blocking(move || {
        let msg = if error.contains("is busy") || error.contains("target is busy") {
            t!(
                "{}: Device is busy. Close any open files and try again.",
                device_label
            )
        } else {
            format!("{}: {}", device_label, error)
        };

        if let Err(e) = Notification::new()
            .summary(&t!("Unmount Failed"))
            .body(&msg)
            .icon("dialog-error")
            .urgency(Urgency::Critical)
            .timeout(5000)
            .show()
        {
            tracing::error!("Failed to show notification: {}", e);
        }
    })
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to spawn notification task: {}", e));
}

pub async fn notify_device_added(device_label: String) {
    tokio::task::spawn_blocking(move || {
        if let Err(e) = Notification::new()
            .summary(&t!("Device Detected"))
            .body(&t!("{} connected", device_label))
            .icon("drive-removable-media")
            .urgency(Urgency::Low)
            .timeout(2000)
            .show()
        {
            tracing::error!("Failed to show notification: {}", e);
        }
    })
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to spawn notification task: {}", e));
}

#[allow(dead_code)]
pub async fn notify_encrypted_device(device_label: String) {
    tokio::task::spawn_blocking(move || {
        if let Err(e) = Notification::new()
            .summary(&t!("Encrypted device detected"))
            .body(&t!("{} connected (encrypted)", device_label))
            .icon("drive-removable-media")
            .urgency(Urgency::Normal)
            .timeout(3000)
            .show()
        {
            tracing::error!("Failed to show notification: {}", e);
        }
    })
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to spawn notification task: {}", e));
}

#[allow(dead_code)]
pub async fn notify_unlock_success(device_label: String) {
    tokio::task::spawn_blocking(move || {
        if let Err(e) = Notification::new()
            .summary(&t!("Device unlocked"))
            .body(&t!("{} unlocked successfully", device_label))
            .icon("drive-removable-media")
            .urgency(Urgency::Normal)
            .timeout(3000)
            .show()
        {
            tracing::error!("Failed to show notification: {}", e);
        }
    })
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to spawn notification task: {}", e));
}

pub async fn notify_unlock_error(device_label: String, error: String) {
    tokio::task::spawn_blocking(move || {
        if let Err(e) = Notification::new()
            .summary(&t!("Unlock failed"))
            .body(&t!("Failed to unlock {}: {}", device_label, error))
            .icon("dialog-error")
            .urgency(Urgency::Critical)
            .timeout(5000)
            .show()
        {
            tracing::error!("Failed to show notification: {}", e);
        }
    })
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to spawn notification task: {}", e));
}
