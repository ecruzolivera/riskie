use anyhow::Result;
use futures::StreamExt;
use std::sync::{Arc, RwLock};
use tracing::{info, error};
use zbus::Connection;

mod udisks2;
mod tray;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting riskie daemon...");
    
    let connection = Connection::system().await?;
    info!("Connected to system D-Bus");
    
    let udisks2_client = udisks2::Client::new(&connection).await?;
    info!("Connected to udisks2");
    
    let devices: Arc<RwLock<Vec<udisks2::Device>>> = Arc::new(RwLock::new(Vec::new()));
    
    {
        let mut devices_guard = devices.write().unwrap();
        let all_devices = udisks2_client.enumerate_devices().await?;
        for device in all_devices {
            if device.is_removable() {
                info!("Found removable device: {} ({})", device.block_device, device.label);
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
                                if !device.is_mounted() {
                                    info!("Automounting device: {} ({})", device.block_device, device.label);
                                    if let Err(e) = client.mount_device(device.object_path.clone()).await {
                                        error!("Failed to mount {}, {} @ {}: {}", device.label, device.block_device, device.object_path, e);
                                    } else {
                                        info!("Successfully mounted {}", device.block_device);
                                    }
                                }
                                {
                                    let mut guard = devices.write().unwrap();
                                    guard.push(device.clone());
                                }
                                let _ = handle.update(|tray| {
                                    let guard = devices.read().unwrap();
                                    tray.devices = Arc::new(RwLock::new(guard.clone()));
                                }).await;
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
                            let mut guard = devices.write().unwrap();
                            guard.retain(|d| d.object_path != path);
                        }
                        let _ = handle.update(|tray| {
                            let guard = devices.read().unwrap();
                            tray.devices = Arc::new(RwLock::new(guard.clone()));
                        }).await;
                    }
                    Err(e) => error!("Error receiving device removed event: {}", e),
                }
            }
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    tray::TrayCommand::Mount(path) => {
                        info!("Mount command for: {}", path);
                        if let Err(e) = client.mount_device(path.clone()).await {
                            error!("Failed to mount: {}", e);
                        }
                        if let Ok(all_devices) = client.enumerate_devices().await {
                            if let Some(device) = all_devices.iter().find(|d| d.object_path == path) {
                                {
                                    let mut guard = devices.write().unwrap();
                                    if let Some(d) = guard.iter_mut().find(|d| d.object_path == path) {
                                        d.filesystem_mount_points = device.filesystem_mount_points.clone();
                                    }
                                }
                                let _ = handle.update(|tray| {
                                    let guard = devices.read().unwrap();
                                    tray.devices = Arc::new(RwLock::new(guard.clone()));
                                }).await;
                            }
                        }
                    }
                    tray::TrayCommand::Unmount(path) => {
                        info!("Unmount command for: {}", path);
                        if let Err(e) = client.unmount_device(path.clone()).await {
                            error!("Failed to unmount: {}", e);
                        }
                        if let Ok(all_devices) = client.enumerate_devices().await {
                            if let Some(device) = all_devices.iter().find(|d| d.object_path == path) {
                                {
                                    let mut guard = devices.write().unwrap();
                                    if let Some(d) = guard.iter_mut().find(|d| d.object_path == path) {
                                        d.filesystem_mount_points = device.filesystem_mount_points.clone();
                                    }
                                }
                                let _ = handle.update(|tray| {
                                    let guard = devices.read().unwrap();
                                    tray.devices = Arc::new(RwLock::new(guard.clone()));
                                }).await;
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