use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use ksni::TrayMethods;
use crate::udisks2::Device;

pub enum TrayCommand {
    Mount(String),
    Unmount(String),
    Exit,
}

pub struct TrayState {
    pub devices: Arc<RwLock<Vec<Device>>>,
    pub command_tx: mpsc::Sender<TrayCommand>,
}

impl ksni::Tray for TrayState {
    fn id(&self) -> String {
        "riskie".into()
    }
    
    fn icon_name(&self) -> String {
        "drive-removable-media".into()
    }
    
    fn title(&self) -> String {
        "Riskie".into()
    }
    
    fn status(&self) -> ksni::Status {
        ksni::Status::Active
    }
    
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        
        let devices_guard = self.devices.blocking_read();
        let mut items: Vec<ksni::MenuItem<Self>> = Vec::new();
        
        if devices_guard.is_empty() {
            items.push(
                StandardItem {
                    label: "No removable devices".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        } else {
            for device in devices_guard.iter() {
                let label = if device.is_mounted() {
                    format!("Unmount {} ({})", device.label, device.block_device)
                } else {
                    format!("Mount {} ({})", device.label, device.block_device)
                };
                
                let object_path = device.object_path.clone();
                let is_mounted = device.is_mounted();
                let tx = self.command_tx.clone();
                
                items.push(
                    StandardItem {
                        label,
                        icon_name: if is_mounted {
                            "drive-removable-media".into()
                        } else {
                            "drive-removable-media".into()
                        },
                        activate: Box::new(move |_tray| {
                            let _ = tx.blocking_send(if is_mounted {
                                TrayCommand::Unmount(object_path.clone())
                            } else {
                                TrayCommand::Mount(object_path.clone())
                            });
                        }),
                        ..Default::default()
                    }
                    .into(),
                );
            }
        }
        
        items.push(MenuItem::Separator);
        
        let tx = self.command_tx.clone();
        items.push(
            StandardItem {
                label: "Exit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(move |_tray| {
                    let _ = tx.blocking_send(TrayCommand::Exit);
                }),
                ..Default::default()
            }
            .into(),
        );
        
        items
    }
}

pub async fn run_tray(
    devices: Arc<RwLock<Vec<Device>>>,
    command_tx: mpsc::Sender<TrayCommand>,
) -> Result<ksni::Handle<TrayState>, ksni::Error> {
    let tray = TrayState {
        devices,
        command_tx,
    };
    
    let handle = tray.spawn().await?;
    
    Ok(handle)
}