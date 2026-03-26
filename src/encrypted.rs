use anyhow::Result;
use std::collections::HashMap;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

#[zbus::proxy(
    interface = "org.freedesktop.UDisks2.Encrypted",
    default_service = "org.freedesktop.UDisks2"
)]
trait Encrypted {
    async fn unlock(
        &self,
        passphrase: &str,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> Result<OwnedObjectPath>;

    async fn lock(&self, options: HashMap<&str, zbus::zvariant::Value<'_>>) -> Result<()>;
}

/// Unlock an encrypted device and return the cleartext device object path
#[allow(dead_code)]
pub async fn unlock_device(
    connection: &Connection,
    object_path: String,
    passphrase: String,
) -> Result<String> {
    let path: ObjectPath<'static> = object_path.try_into()?;
    let encrypted = EncryptedProxy::new(connection, path).await?;
    let options = HashMap::new();
    let cleartext_path = encrypted.unlock(&passphrase, options).await?;
    Ok(cleartext_path.to_string())
}

/// Lock an encrypted device
#[allow(dead_code)]
pub async fn lock_device(connection: &Connection, object_path: String) -> Result<()> {
    let path: ObjectPath<'static> = object_path.try_into()?;
    let encrypted = EncryptedProxy::new(connection, path).await?;
    let options = HashMap::new();
    encrypted.lock(options).await
}
