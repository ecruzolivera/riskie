use std::sync::{Arc, RwLock};

use anyhow::Result;
use futures::StreamExt;
use tracing::{error, info};
use zbus::Connection;

mod notify;
mod tray;
mod udisks2;

async fn update_tray_devices(
    handle: &tray::TrayHandle,
    devices: &Arc<RwLock<Vec<udisks2::Device>>>,
) {
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
                        if let Some(device) = all_devices.iter().find(|d| d.object_path == path)
                            && device.is_removable()
                        {
                            let device_label = if device.label.is_empty() {
                                device.block_device.clone()
                            } else {
                                device.label.clone()
                            };

                            notify::notify_device_added(device_label.clone()).await;

                            if !device.is_mounted() {
                                info!("Automounting device: {} ({})", device.block_device, device.label);
                                match client.mount_device(device.object_path.clone()).await {
                                    Ok(mount_point) => {
                                        info!("Successfully mounted {} at {}", device.block_device, mount_point);
                                        notify::notify_mount_success(device_label.clone(), mount_point).await;
                                    }
                                    Err(e) => {
                                        let error_msg = e.to_string();
                                        error!("Failed to mount {}, {} @ {}: {}", device.label, device.block_device, device.object_path, error_msg);
                                        notify::notify_mount_error(device_label.clone(), error_msg).await;
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
                                notify::notify_mount_success(device_label.clone(), mount_point).await;
                            }
                            Err(e) => {
                                let error_msg = e.to_string();
                                error!("Failed to mount {}: {}", path, error_msg);
                                notify::notify_mount_error(device_label.clone(), error_msg).await;
                            }
                        }

                        if let Ok(all_devices) = client.enumerate_devices().await
                            && let Some(device) = all_devices.iter().find(|d| d.object_path == path)
                        {
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
                                notify::notify_unmount_success(device_label.clone()).await;
                            }
                            Err(e) => {
                                let error_msg = e.to_string();
                                error!("Failed to unmount {}: {}", path, error_msg);
                                notify::notify_unmount_error(device_label.clone(), error_msg).await;
                            }
                        }

                        if let Ok(all_devices) = client.enumerate_devices().await
                            && let Some(device) = all_devices.iter().find(|d| d.object_path == path)
                        {
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
                    tray::TrayCommand::EjectAll(drive_path) => {
                        info!("Eject all command for drive: {}", drive_path);
                        let partitions_to_unmount: Vec<(String, String)> = {
                            let guard = match devices.read() {
                                Ok(g) => g,
                                Err(e) => {
                                    error!("Failed to acquire read lock: {}", e);
                                    continue;
                                }
                            };
                            guard.iter()
                                .filter(|d| d.drive_id() == drive_path && d.is_mounted())
                                .map(|d| {
                                    let label = if d.label.is_empty() { d.block_device.clone() } else { d.label.clone() };
                                    (d.object_path.clone(), label)
                                })
                                .collect()
                        };

                        if partitions_to_unmount.is_empty() {
                            info!("No mounted partitions to eject for drive: {}", drive_path);
                            continue;
                        }

                        let mut all_success = true;
                        for (partition_path, partition_label) in &partitions_to_unmount {
                            info!("Unmounting partition: {}", partition_path);
                            match client.unmount_device(partition_path.clone()).await {
                                Ok(()) => {
                                    info!("Successfully unmounted {}", partition_path);
                                }
                                Err(e) => {
                                    let error_msg = e.to_string();
                                    error!("Failed to unmount {}: {}", partition_path, error_msg);
                                    notify::notify_unmount_error(partition_label.clone(), error_msg).await;
                                    all_success = false;
                                }
                            }
                        }

                        if all_success && !partitions_to_unmount.is_empty() {
                            let drive_label = partitions_to_unmount.first()
                                .map(|(_, l)| l.clone())
                                .unwrap_or_else(|| drive_path.clone());
                            notify::notify_unmount_success(format!("{} (all partitions)", drive_label)).await;
                        }

                        if let Ok(all_devices) = client.enumerate_devices().await {
                            {
                                let mut guard = match devices.write() {
                                    Ok(g) => g,
                                    Err(e) => {
                                        error!("Failed to acquire write lock: {}", e);
                                        continue;
                                    }
                                };
                                for device in &all_devices {
                                    if let Some(d) = guard.iter_mut().find(|d| d.object_path == device.object_path) {
                                        d.filesystem_mount_points = device.filesystem_mount_points.clone();
                                    }
                                }
                            }
                            update_tray_devices(&handle, &devices).await;
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
