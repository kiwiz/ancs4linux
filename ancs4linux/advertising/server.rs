use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::dbus_interface;
use zvariant::ObjectPath;

use crate::advertising::advertisement::AdvertisingManager;
use crate::advertising::pairing::PairingManager;
use crate::common::apis::AdvertisingAPI;
use crate::common::dbus::Str;

#[dbus_interface(name = "ancs4linux.Advertising")]
struct AdvertisingServer {
    pairing_manager: Arc<Mutex<PairingManager>>,
    advertising_manager: Arc<Mutex<AdvertisingManager>>,
}

impl AdvertisingServer {
    fn new(
        pairing_manager: Arc<Mutex<PairingManager>>,
        advertising_manager: Arc<Mutex<AdvertisingManager>>,
    ) -> Self {
        Self {
            pairing_manager,
            advertising_manager,
        }
    }

    #[dbus_interface(property)]
    async fn get_all_hci(&self) -> Vec<Str> {
        self.advertising_manager.lock().await.get_all_hci_addresses().await
    }

    #[dbus_interface(property)]
    async fn enable_advertising(&self, hci_address: Str, name: Str) {
        self.advertising_manager
            .lock()
            .await
            .enable_advertising(&hci_address, name.into())
            .await;
    }

    #[dbus_interface(property)]
    async fn disable_advertising(&self, hci_address: Str) {
        self.advertising_manager
            .lock()
            .await
            .disable_advertising(&hci_address)
            .await;
    }

    #[dbus_interface(property)]
    async fn enable_pairing(&self) {
        self.pairing_manager.lock().await.enable().await;
    }

    #[dbus_interface(property)]
    async fn disable_pairing(&self) {
        self.pairing_manager.lock().await.disable().await;
    }

    #[dbus_interface(signal)]
    async fn pairing_code(&self, pin: Str) {}
}
