use std::sync::{Arc, RwLock};

use anyhow::Result;
use futures::StreamExt;
use tracing::{error, info};
use zbus::Connection;

mod notify;
mod tray;
mod udisks2;

async fn update_tray_devices(handle: &tray::TrayHandle, devices: &Arc<RwLock<Vec<udisks2::Device>>>) {
    let devices_clone = {
        let guard = match devices.read() {
            Ok(g) => g,
            Err(e) => {
                error!("Failed to acquire read lock: {}", e);
                return;
            }
        };
        guard.clone()
    };

    if handle
        .update(move |tray| {
            tray.devices = Arc::new(RwLock::new(devices_clone));
        })
        .await
        .is_none()
    {
        error!("Failed to update tray: tray service unavailable");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting riskie daemon...");

    let connection = Connection::system().await?;
    info!("Connected to system D-Bus");

    let udisks2_client = udisks2::Client::new(&connection).await?;
    info!("Connected to udisks2");

    let devices: Arc<RwLock<Vec<udisks2::Device>>> = Arc::new(RwLock::new(Vec::new()));

    let all_devices = udisks2_client.enumerate_devices().await?;
    {
        let mut devices_guard = match devices.write() {
            Ok(g) => g,
            Err(e) => {
                error!("Failed to acquire write lock: {}", e);
                return Err(anyhow::anyhow!("Failed to acquire write lock: {}", e));
            }
        };
        for device in all_devices {
            if device.is_removable() {
                info!(
                    "Found removable device: {} ({})",
                    device.block_device, device.label
                );
                devices_guard.push(device);
            }
        }
    }

    let client = udisks2::Client::new(&connection).await?;
    let mut device_added = client.subscribe_device_added().await?;
    let mut device_removed = client.subscribe_device_removed().await?;

    let (command_tx, mut command_rx) = tokio::sync::mpsc::channel::<tray::TrayCommand>(16);

    let handle = tray::run_tray(devices.clone(), command_tx.clone()).await?;
    info!("System tray initialized");

    info!("Listening for device events...");

    loop {
        tokio::select! {
            Some(result) = device_added.next() => {
                match result {
                    Ok(path) => {
                        info!("Device added: {}", path);
                        let all_devices = client.enumerate_devices().await?;
                        if let Some(device) = all_devices.iter().find(|d| d.object_path == path) {
                            if device.is_removable() {
                                let device_label = if device.label.is_empty() {
                                    device.block_device.clone()
                                } else {
                                    device.label.clone()
                                };

                                notify::notify_device_added(&device_label);

                                if !device.is_mounted() {
                                    info!("Automounting device: {} ({})", device.block_device, device.label);
                                    match client.mount_device(device.object_path.clone()).await {
                                        Ok(mount_point) => {
                                            info!("Successfully mounted {} at {}", device.block_device, mount_point);
                                            notify::notify_mount_success(&device_label, &mount_point);
                                        }
                                        Err(e) => {
                                            let error_msg = e.to_string();
                                            error!("Failed to mount {}, {} @ {}: {}", device.label, device.block_device, device.object_path, error_msg);
                                            notify::notify_mount_error(&device_label, &error_msg);
                                        }
                                    }
                                }
                                {
                                    let mut guard = match devices.write() {
                                        Ok(g) => g,
                                        Err(e) => {
                                            error!("Failed to acquire write lock: {}", e);
                                            continue;
                                        }
                                    };
                                    guard.push(device.clone());
                                }
                                update_tray_devices(&handle, &devices).await;
                            }
                        }
                    }
                    Err(e) => error!("Error receiving device added event: {}", e),
                }
            }
            Some(result) = device_removed.next() => {
                match result {
                    Ok(path) => {
                        info!("Device removed: {}", path);
                        {
                            let mut guard = match devices.write() {
                                Ok(g) => g,
                                Err(e) => {
                                    error!("Failed to acquire write lock: {}", e);
                                    continue;
                                }
                            };
                            guard.retain(|d| d.object_path != path);
                        }
                        update_tray_devices(&handle, &devices).await;
                    }
                    Err(e) => error!("Error receiving device removed event: {}", e),
                }
            }
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    tray::TrayCommand::Mount(path) => {
                        info!("Mount command for: {}", path);
                        let device_label = {
                            let guard = devices.read().ok();
                            guard.and_then(|g| {
                                g.iter().find(|d| d.object_path == path).map(|d| {
                                    if d.label.is_empty() { d.block_device.clone() } else { d.label.clone() }
                                })
                            }).unwrap_or_else(|| path.clone())
                        };

                        match client.mount_device(path.clone()).await {
                            Ok(mount_point) => {
                                info!("Successfully mounted {} at {}", path, mount_point);
                                notify::notify_mount_success(&device_label, &mount_point);
                            }
                            Err(e) => {
                                let error_msg = e.to_string();
                                error!("Failed to mount {}: {}", path, error_msg);
                                notify::notify_mount_error(&device_label, &error_msg);
                            }
                        }

                        if let Ok(all_devices) = client.enumerate_devices().await {
                            if let Some(device) = all_devices.iter().find(|d| d.object_path == path) {
                                {
                                    let mut guard = match devices.write() {
                                        Ok(g) => g,
                                        Err(e) => {
                                            error!("Failed to acquire write lock: {}", e);
                                            continue;
                                        }
                                    };
                                    if let Some(d) = guard.iter_mut().find(|d| d.object_path == path) {
                                        d.filesystem_mount_points = device.filesystem_mount_points.clone();
                                    }
                                }
                                update_tray_devices(&handle, &devices).await;
                            }
                        }
                    }
                    tray::TrayCommand::Unmount(path) => {
                        info!("Unmount command for: {}", path);
                        let device_label = {
                            let guard = devices.read().ok();
                            guard.and_then(|g| {
                                g.iter().find(|d| d.object_path == path).map(|d| {
                                    if d.label.is_empty() { d.block_device.clone() } else { d.label.clone() }
                                })
                            }).unwrap_or_else(|| path.clone())
                        };

                        match client.unmount_device(path.clone()).await {
                            Ok(()) => {
                                info!("Successfully unmounted {}", path);
                                notify::notify_unmount_success(&device_label);
                            }
                            Err(e) => {
                                let error_msg = e.to_string();
                                error!("Failed to unmount {}: {}", path, error_msg);
                                notify::notify_unmount_error(&device_label, &error_msg);
                            }
                        }

                        if let Ok(all_devices) = client.enumerate_devices().await {
                            if let Some(device) = all_devices.iter().find(|d| d.object_path == path) {
                                {
                                    let mut guard = match devices.write() {
                                        Ok(g) => g,
                                        Err(e) => {
                                            error!("Failed to acquire write lock: {}", e);
                                            continue;
                                        }
                                    };
                                    if let Some(d) = guard.iter_mut().find(|d| d.object_path == path) {
                                        d.filesystem_mount_points = device.filesystem_mount_points.clone();
                                    }
                                }
                                update_tray_devices(&handle, &devices).await;
                            }
                        }
                    }
                    tray::TrayCommand::Exit => {
                        info!("Exit requested");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}