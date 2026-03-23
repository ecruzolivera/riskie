use notify_rust::{Notification, Urgency};

pub fn notify_mount_success(device_label: &str, mount_point: &str) {
    if let Err(e) = Notification::new()
        .summary("Device Mounted")
        .body(&format!("{}\nMounted at {}", device_label, mount_point))
        .icon("drive-removable-media")
        .urgency(Urgency::Normal)
        .timeout(3000)
        .show()
    {
        tracing::error!("Failed to show notification: {}", e);
    }
}

pub fn notify_mount_error(device_label: &str, error: &str) {
    if let Err(e) = Notification::new()
        .summary("Mount Failed")
        .body(&format!("{}: {}", device_label, error))
        .icon("dialog-error")
        .urgency(Urgency::Critical)
        .timeout(5000)
        .show()
    {
        tracing::error!("Failed to show notification: {}", e);
    }
}

pub fn notify_unmount_success(device_label: &str) {
    if let Err(e) = Notification::new()
        .summary("Device Unmounted")
        .body(&format!("{} safely removed", device_label))
        .icon("drive-removable-media")
        .urgency(Urgency::Normal)
        .timeout(3000)
        .show()
    {
        tracing::error!("Failed to show notification: {}", e);
    }
}

pub fn notify_unmount_error(device_label: &str, error: &str) {
    let msg = if error.contains("is busy") || error.contains("target is busy") {
        format!(
            "{}: Device is busy. Close any open files and try again.",
            device_label
        )
    } else {
        format!("{}: {}", device_label, error)
    };

    if let Err(e) = Notification::new()
        .summary("Unmount Failed")
        .body(&msg)
        .icon("dialog-error")
        .urgency(Urgency::Critical)
        .timeout(5000)
        .show()
    {
        tracing::error!("Failed to show notification: {}", e);
    }
}

pub fn notify_device_added(device_label: &str) {
    if let Err(e) = Notification::new()
        .summary("Device Detected")
        .body(&format!("{} connected", device_label))
        .icon("drive-removable-media")
        .urgency(Urgency::Low)
        .timeout(2000)
        .show()
    {
        tracing::error!("Failed to show notification: {}", e);
    }
}
