use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::dbus_interface;
use zvariant::ObjectPath;

use crate::common::dbus::{Bool, Str, SystemBus, UInt16, UInt32};

#[derive(Serialize, Deserialize)]
pub struct ShowNotificationData {
    pub device_name: String,
    pub device_handle: String,
    pub app_id: String,
    pub app_name: String,
    pub id: u32,
    pub title: String,
    pub body: String,
    pub positive_action: Option<String>,
    pub negative_action: Option<String>,
}

impl ShowNotificationData {
    pub fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn parse(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }
}

#[dbus_interface(name = "ancs4linux.Observer")]
pub trait ObserverAPI {
    fn invoke_device_action(
        &self,
        device_handle: Str,
        notification_id: UInt32,
        is_positive: Bool,
    );

    #[dbus_interface(signal)]
    fn show_notification(&self, data: String);

    #[dbus_interface(signal)]
    fn dismiss_notification(&self, id: UInt32);
}

#[dbus_interface(name = "ancs4linux.Advertising")]
pub trait AdvertisingAPI {
    fn get_all_hci(&self) -> Vec<Str>;

    fn enable_advertising(&self, hci_address: Str, name: Str);

    fn disable_advertising(&self, hci_address: Str);

    fn enable_pairing(&self);

    fn disable_pairing(&self);

    #[dbus_interface(signal)]
    fn pairing_code(&self, pin: String);
}

#[dbus_interface(name = "org.bluez.Agent1")]
pub trait PairingAgentAPI {
    fn release(&self);

    fn request_pin_code(&self, device: ObjectPath<'_>) -> Result<Str, ()>;

    fn display_pin_code(&self, device: ObjectPath<'_>, pincode: Str) -> Result<(), ()>;

    fn request_passkey(&self, device: ObjectPath<'_>) -> Result<UInt32, ()>;

    fn display_passkey(
        &self,
        device: ObjectPath<'_>,
        passkey: UInt32,
        entered: UInt16,
    ) -> Result<(), ()>;

    fn request_confirmation(&self, device: ObjectPath<'_>, passkey: UInt32) -> Result<(), ()>;

    fn request_authorization(&self, device: ObjectPath<'_>) -> Result<(), ()>;

    fn authorize_service(&self, device: ObjectPath<'_>, uuid: Str) -> Result<(), ()>;

    fn cancel(&self);
}
