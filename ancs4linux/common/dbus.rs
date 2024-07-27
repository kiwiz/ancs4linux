use async_trait::async_trait;
use zbus::{Connection, dbus_interface, dbus_proxy, fdo::Result};
use zbus::zvariant::ObjectPath;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait MessageBus {
    async fn publish_object(&self, address: &str, object: Arc<dyn zbus::Interface + Send + Sync>) -> Result<()>;
    async fn register_service(&self, name: &str) -> Result<()>;
    async fn get_proxy<'a>(&self, name: &str, address: &str) -> Result<zbus::Proxy<'a>>;
}

pub struct SystemBus {
    connection: Arc<Mutex<Connection>>,
}

impl SystemBus {
    pub async fn new() -> Result<Self> {
        let connection = Connection::system().await?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

#[async_trait]
impl MessageBus for SystemBus {
    async fn publish_object(&self, address: &str, object: Arc<dyn zbus::Interface + Send + Sync>) -> Result<()> {
        let connection = self.connection.lock().await;
        connection.object_server().at(ObjectPath::try_from(address)?, object).await?;
        Ok(())
    }

    async fn register_service(&self, name: &str) -> Result<()> {
        let connection = self.connection.lock().await;
        connection.request_name(name).await?;
        Ok(())
    }

    async fn get_proxy<'a>(&self, name: &str, address: &str) -> Result<zbus::Proxy<'a>> {
        let connection = self.connection.lock().await;
        let proxy = zbus::Proxy::new(&*connection, name, ObjectPath::try_from(address)?).await?;
        Ok(proxy)
    }
}

pub struct SessionBus {
    connection: Arc<Mutex<Connection>>,
}

impl SessionBus {
    pub async fn new() -> Result<Self> {
        let connection = Connection::session().await?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

#[async_trait]
impl MessageBus for SessionBus {
    async fn publish_object(&self, address: &str, object: Arc<dyn zbus::Interface + Send + Sync>) -> Result<()> {
        let connection = self.connection.lock().await;
        connection.object_server().at(ObjectPath::try_from(address)?, object).await?;
        Ok(())
    }

    async fn register_service(&self, name: &str) -> Result<()> {
        let connection = self.connection.lock().await;
        connection.request_name(name).await?;
        Ok(())
    }

    async fn get_proxy<'a>(&self, name: &str, address: &str) -> Result<zbus::Proxy<'a>> {
        let connection = self.connection.lock().await;
        let proxy = zbus::Proxy::new(&*connection, name, ObjectPath::try_from(address)?).await?;
        Ok(proxy)
    }
}

#[derive(Debug)]
pub struct PairingRejected;

impl std::fmt::Display for PairingRejected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pairing rejected")
    }
}

impl std::error::Error for PairingRejected {}
