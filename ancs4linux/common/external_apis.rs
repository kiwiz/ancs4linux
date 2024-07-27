use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zbus::dbus_interface;
use zvariant::{ObjectPath, OwnedValue, Value};

#[async_trait]
pub trait PropertiesAPI {
    async fn get(&self, interface: &str, name: &str) -> OwnedValue;
    async fn get_all(&self, interface: &str) -> HashMap<String, OwnedValue>;
    async fn set(&self, interface: &str, name: &str, value: OwnedValue);
}

#[async_trait]
pub trait ObjectManagerAPI {
    async fn get_managed_objects(&self) -> HashMap<ObjectPath<'_>, HashMap<String, HashMap<String, OwnedValue>>>;
}

#[async_trait]
pub trait NotificationAPI {
    async fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: Vec<OwnedValue>,
        expire_timeout: i32,
    ) -> u32;

    async fn close_notification(&self, id: u32);
}

#[async_trait]
pub trait BluezRootAPI: ObjectManagerAPI {
    async fn connect() -> Self;
}

#[async_trait]
pub trait BluezAgentManagerAPI {
    async fn register_agent(&self, agent: &str, capability: &str);
    async fn request_default_agent(&self, agent: &str);
    async fn unregister_agent(&self, agent: &str);
}

#[async_trait]
pub trait BluezDeviceAPI: PropertiesAPI {
    async fn connect(&self);
}

#[async_trait]
pub trait BluezGattCharacteristicAPI: PropertiesAPI {
    async fn read_value(&self, options: HashMap<String, OwnedValue>) -> Vec<u8>;
    async fn write_value(&self, value: Vec<u8>, options: HashMap<String, OwnedValue>);
    async fn start_notify(&self);
    async fn stop_notify(&self);
}
