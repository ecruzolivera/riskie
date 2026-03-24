use anyhow::Result;
use futures::StreamExt;
use zbus::fdo::ObjectManagerProxy;
use zbus::zvariant::{ObjectPath, OwnedValue};
use zbus::Connection;

/// Represents a block device from udisks2
#[derive(Debug, Clone)]
pub struct Device {
    pub object_path: String,
    pub block_device: String,
    pub label: String,
    #[allow(dead_code)]
    pub size: u64,
    pub filesystem_mount_points: Vec<String>,
    pub hint_auto: bool,
    pub hint_system: bool,
    pub drive: Option<String>,
}

impl Device {
    pub fn is_removable(&self) -> bool {
        self.hint_auto && !self.hint_system
    }

    pub fn is_mounted(&self) -> bool {
        !self.filesystem_mount_points.is_empty()
    }

    pub fn drive_id(&self) -> &str {
        self.drive.as_deref().unwrap_or(&self.object_path)
    }
}

#[zbus::proxy(
    interface = "org.freedesktop.UDisks2.Filesystem",
    default_service = "org.freedesktop.UDisks2"
)]
trait Filesystem {
    async fn mount(
        &self,
        options: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> Result<String>;

    async fn unmount(
        &self,
        options: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> Result<()>;
}

/// udisks2 client for interacting with the UDisks2 daemon
pub struct Client {
    connection: Connection,
}

impl Client {
    /// Create a new udisks2 client
    pub async fn new(connection: &Connection) -> Result<Self> {
        Ok(Self {
            connection: connection.clone(),
        })
    }

    /// Enumerate all block devices
    pub async fn enumerate_devices(&self) -> Result<Vec<Device>> {
        let object_manager = ObjectManagerProxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            "/org/freedesktop/UDisks2",
        )
        .await?;

        let managed_objects = object_manager.get_managed_objects().await?;

        let mut devices = Vec::new();

        for (object_path, interfaces) in managed_objects {
            // Check if this object has the Block interface
            if let Some(block_props) = interfaces.get("org.freedesktop.UDisks2.Block") {
                let block_device =
                    get_property_byte_array(block_props, "Device").unwrap_or_default();
                let label = get_property_string(block_props, "IdLabel").unwrap_or_default();
                let size = get_property_u64(block_props, "Size").unwrap_or(0);
                let hint_auto = get_property_bool(block_props, "HintAuto").unwrap_or(false);
                let hint_system = get_property_bool(block_props, "HintSystem").unwrap_or(true);
                let drive = get_property_object_path(block_props, "Drive");

                let filesystem_mount_points = interfaces
                    .get("org.freedesktop.UDisks2.Filesystem")
                    .map(|fs_props| get_property_mount_points(fs_props).unwrap_or_default())
                    .unwrap_or_default();

                devices.push(Device {
                    object_path: object_path.to_string(),
                    block_device,
                    label,
                    size,
                    filesystem_mount_points,
                    hint_auto,
                    hint_system,
                    drive,
                });
            }
        }

        Ok(devices)
    }

    /// Subscribe to device added events
    pub async fn subscribe_device_added(
        &self,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>> {
        let object_manager = ObjectManagerProxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            "/org/freedesktop/UDisks2",
        )
        .await?;

        let stream = async_stream::stream! {
            let mut stream = object_manager.receive_interfaces_added().await?;

            while let Some(signal) = stream.next().await {
                if let Ok(args) = signal.args() {
                    // Check if this has the Block interface
                    if args.interfaces_and_properties.contains_key("org.freedesktop.UDisks2.Block") {
                        yield Ok(args.object_path.to_string());
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Subscribe to device removed events
    pub async fn subscribe_device_removed(
        &self,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>> {
        let object_manager = ObjectManagerProxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            "/org/freedesktop/UDisks2",
        )
        .await?;

        let stream = async_stream::stream! {
            let mut stream = object_manager.receive_interfaces_removed().await?;

            while let Some(signal) = stream.next().await {
                if let Ok(args) = signal.args() {
                    // Check if Block interface was removed
                    if args.interfaces.contains(&"org.freedesktop.UDisks2.Block") {
                        yield Ok(args.object_path.to_string());
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Mount a device and return the mount point
    pub async fn mount_device(&self, object_path: String) -> Result<String> {
        let path: ObjectPath<'static> = ObjectPath::try_from(object_path)?;
        let filesystem = FilesystemProxy::new(&self.connection, path).await?;
        let options = std::collections::HashMap::new();
        filesystem.mount(options).await
    }

    /// Unmount a device
    pub async fn unmount_device(&self, object_path: String) -> Result<()> {
        let path: ObjectPath<'static> = ObjectPath::try_from(object_path)?;
        let filesystem = FilesystemProxy::new(&self.connection, path).await?;
        let options = std::collections::HashMap::new();
        filesystem.unmount(options).await
    }
}

// Helper functions to extract properties from OwnedValue

fn get_property_string(
    props: &std::collections::HashMap<String, OwnedValue>,
    key: &str,
) -> Option<String> {
    props.get(key).and_then(|v| {
        if let Ok(s) = v.downcast_ref::<&str>() {
            Some(s.to_string())
        } else if let Ok(s) = v.downcast_ref::<String>() {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn get_property_byte_array(
    props: &std::collections::HashMap<String, OwnedValue>,
    key: &str,
) -> Option<String> {
    props.get(key).and_then(|v| {
        if let Ok(arr) = v.downcast_ref::<zbus::zvariant::Array>() {
            let mut bytes = Vec::new();
            for item in arr.iter() {
                if let Ok(b) = item.downcast_ref::<u8>() {
                    if b != 0 {
                        bytes.push(b);
                    }
                }
            }
            if !bytes.is_empty() {
                let s: String = bytes.iter().map(|&b| b as char).collect();
                return Some(s);
            }
        }
        None
    })
}

fn get_property_u64(
    props: &std::collections::HashMap<String, OwnedValue>,
    key: &str,
) -> Option<u64> {
    props.get(key).and_then(|v| {
        if let Ok(n) = v.downcast_ref::<u64>() {
            Some(n)
        } else if let Ok(n) = v.downcast_ref::<i64>() {
            Some(n as u64)
        } else {
            None
        }
    })
}

fn get_property_bool(
    props: &std::collections::HashMap<String, OwnedValue>,
    key: &str,
) -> Option<bool> {
    props.get(key).and_then(|v| v.downcast_ref::<bool>().ok())
}

fn get_property_mount_points(
    props: &std::collections::HashMap<String, OwnedValue>,
) -> Option<Vec<String>> {
    props.get("MountPoints").and_then(|v| {
        if let Ok(paths) = v.downcast_ref::<zbus::zvariant::Array>() {
            let mut result = Vec::new();
            for item in paths.iter() {
                if let Ok(bytes) = item.downcast_ref::<zbus::zvariant::Array>() {
                    let path: String = bytes
                        .iter()
                        .filter_map(|b| b.downcast_ref::<u8>().ok())
                        .map(|b| b as char)
                        .collect();
                    if !path.is_empty() {
                        result.push(path);
                    }
                }
            }
            if !result.is_empty() {
                return Some(result);
            }
        }
        None
    })
}

fn get_property_object_path(
    props: &std::collections::HashMap<String, OwnedValue>,
    key: &str,
) -> Option<String> {
    props.get(key).and_then(|v| {
        if let Ok(path) = v.downcast_ref::<zbus::zvariant::ObjectPath>() {
            Some(path.to_string())
        } else if let Ok(s) = v.downcast_ref::<&str>() {
            Some(s.to_string())
        } else if let Ok(s) = v.downcast_ref::<String>() {
            Some(s.clone())
        } else {
            None
        }
    })
}
