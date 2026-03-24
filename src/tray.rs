use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use ksni::TrayMethods;
use tokio::sync::mpsc;

use crate::udisks2::Device;

pub enum TrayCommand {
    Mount(String),
    Unmount(String),
    EjectAll(String),
    Exit,
}

pub struct TrayState {
    pub devices: Arc<RwLock<Vec<Device>>>,
    pub command_tx: mpsc::Sender<TrayCommand>,
}

impl ksni::Tray for TrayState {
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        "riskie".into()
    }

    fn icon_name(&self) -> String {
        "drive-removable-media-usb".into()
    }

    fn title(&self) -> String {
        "Riskie".into()
    }

    fn status(&self) -> ksni::Status {
        ksni::Status::Active
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        let devices_guard = match self.devices.read() {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Failed to acquire read lock for menu: {}", e);
                return vec![
                    StandardItem {
                        label: "Error: unable to read devices".into(),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                ];
            }
        };

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
            let mut drive_groups: HashMap<String, Vec<&Device>> = HashMap::new();
            for device in devices_guard.iter() {
                let drive_id = device.drive_id().to_string();
                drive_groups.entry(drive_id).or_default().push(device);
            }

            for (drive_id, partitions) in drive_groups {
                let drive_label = partitions
                    .first()
                    .map(|d| {
                        if d.label.is_empty() {
                            d.block_device.clone()
                        } else {
                            d.label.clone()
                        }
                    })
                    .unwrap_or_else(|| "Unknown Drive".to_string());

                let mounted_count = partitions.iter().filter(|p| p.is_mounted()).count();
                let total_count = partitions.len();
                let any_mounted = mounted_count > 0;

                items.push(
                    StandardItem {
                        label: drive_label.clone(),
                        icon_name: "drive-removable-media".into(),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                );

                if total_count == 1 {
                    let device = partitions[0];
                    let part_label = if device.label.is_empty() {
                        device.block_device.clone()
                    } else {
                        device.label.clone()
                    };
                    let label = if device.is_mounted() {
                        format!("  Unmount {}", part_label)
                    } else {
                        format!("  Mount {}", part_label)
                    };

                    let object_path = device.object_path.clone();
                    let is_mounted = device.is_mounted();
                    let tx = self.command_tx.clone();

                    items.push(
                        StandardItem {
                            label,
                            icon_name: "drive-harddisk".into(),
                            activate: Box::new(move |_tray| {
                                if let Err(e) = tx.try_send(if is_mounted {
                                    TrayCommand::Unmount(object_path.clone())
                                } else {
                                    TrayCommand::Mount(object_path.clone())
                                }) {
                                    tracing::error!("Failed to send mount/unmount command: {}", e);
                                }
                            }),
                            ..Default::default()
                        }
                        .into(),
                    );

                    if any_mounted {
                        let drive_id_clone = drive_id.clone();
                        let tx = self.command_tx.clone();
                        let label = format!("  Eject {}", drive_label);

                        items.push(
                            StandardItem {
                                label,
                                icon_name: "media-eject".into(),
                                activate: Box::new(move |_tray| {
                                    if let Err(e) =
                                        tx.try_send(TrayCommand::EjectAll(drive_id_clone.clone()))
                                    {
                                        tracing::error!("Failed to send eject command: {}", e);
                                    }
                                }),
                                ..Default::default()
                            }
                            .into(),
                        );
                    }
                } else {
                    for partition in &partitions {
                        let part_label = if partition.label.is_empty() {
                            partition.block_device.clone()
                        } else {
                            partition.label.clone()
                        };
                        let label = if partition.is_mounted() {
                            format!("  Unmount {}", part_label)
                        } else {
                            format!("  Mount {}", part_label)
                        };

                        let object_path = partition.object_path.clone();
                        let is_mounted = partition.is_mounted();
                        let tx = self.command_tx.clone();

                        items.push(
                            StandardItem {
                                label,
                                icon_name: "drive-harddisk".into(),
                                activate: Box::new(move |_tray| {
                                    if let Err(e) = tx.try_send(if is_mounted {
                                        TrayCommand::Unmount(object_path.clone())
                                    } else {
                                        TrayCommand::Mount(object_path.clone())
                                    }) {
                                        tracing::error!(
                                            "Failed to send mount/unmount command: {}",
                                            e
                                        );
                                    }
                                }),
                                ..Default::default()
                            }
                            .into(),
                        );
                    }

                    if any_mounted {
                        let drive_id_clone = drive_id.clone();
                        let label = format!("  Eject {} (unmount all)", drive_label);
                        let tx = self.command_tx.clone();

                        items.push(
                            StandardItem {
                                label,
                                icon_name: "media-eject".into(),
                                activate: Box::new(move |_tray| {
                                    if let Err(e) =
                                        tx.try_send(TrayCommand::EjectAll(drive_id_clone.clone()))
                                    {
                                        tracing::error!("Failed to send eject command: {}", e);
                                    }
                                }),
                                ..Default::default()
                            }
                            .into(),
                        );
                    }
                }

                items.push(MenuItem::Separator);
            }
        }

        items.push(MenuItem::Separator);

        let tx = self.command_tx.clone();
        items.push(
            StandardItem {
                label: "Exit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(move |_tray| {
                    if let Err(e) = tx.try_send(TrayCommand::Exit) {
                        tracing::error!("Failed to send exit command: {}", e);
                    }
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

pub type TrayHandle = ksni::Handle<TrayState>;

pub async fn run_tray(
    devices: Arc<RwLock<Vec<Device>>>,
    command_tx: mpsc::Sender<TrayCommand>,
) -> Result<TrayHandle, ksni::Error> {
    let tray = TrayState {
        devices,
        command_tx,
    };

    let handle = tray.spawn().await?;

    Ok(handle)
}
