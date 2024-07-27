use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::dbus_interface;
use zvariant::{ObjectPath, OwnedValue, Value};

use crate::advertising::pairing::PairingManager;
use crate::common::dbus::{Bool, Byte, Str, SystemBus, UInt16, Variant};
use crate::common::external_apis::BluezRootAPI;

fn array_of_bytes(array: Vec<u8>) -> Variant {
    Variant::new(Value::Array(array.into_iter().map(Byte).collect()))
}

#[dbus_interface(name = "org.bluez.LEAdvertisement1")]
struct AdvertisementData {
    #[dbus_interface(property)]
    fn type_(&self) -> Str {
        "peripheral".into()
    }

    #[dbus_interface(property)]
    fn service_uuids(&self) -> Vec<Str> {
        vec![]
    }

    #[dbus_interface(property)]
    fn include_tx_power(&self) -> Bool {
        true
    }

    #[dbus_interface(property)]
    fn manufacturer_data(&self) -> HashMap<UInt16, Variant> {
        let mut data = HashMap::new();
        data.insert(UInt16(0xFFFF), array_of_bytes(vec![0x50, 0xB0, 0x13, 0xF0]));
        data
    }

    #[dbus_interface(property)]
    fn service_data(&self) -> HashMap<Str, Variant> {
        let mut data = HashMap::new();
        data.insert("9999".into(), array_of_bytes(vec![0x9E, 0x85, 0x39, 0x96]));
        data
    }

    fn release(&self) {}
}

struct HciState {
    name: String,
    powered: bool,
    discoverable: bool,
    pairable: bool,
}

impl HciState {
    fn advertising(name: String) -> Self {
        Self {
            name,
            powered: true,
            discoverable: true,
            pairable: true,
        }
    }

    fn save(hci: &BluezRootAPI) -> Self {
        Self {
            name: hci.alias().unwrap(),
            powered: hci.powered().unwrap(),
            discoverable: hci.discoverable().unwrap(),
            pairable: hci.pairable().unwrap(),
        }
    }

    fn restore_on(&self, hci: &BluezRootAPI) {
        hci.set_powered(self.powered).unwrap();
        hci.set_alias(&self.name).unwrap();
        if self.powered {
            hci.set_pairable(self.pairable).unwrap();
            hci.set_discoverable(self.discoverable).unwrap();
        }
    }
}

struct AdvertisingManager {
    active_advertisements: Arc<Mutex<HashMap<String, HciState>>>,
    pairing_manager: Arc<Mutex<PairingManager>>,
}

impl AdvertisingManager {
    const ADDRESS: &'static str = "/advertisement";

    fn new(pairing_manager: Arc<Mutex<PairingManager>>) -> Self {
        Self {
            active_advertisements: Arc::new(Mutex::new(HashMap::new())),
            pairing_manager,
        }
    }

    async fn register(&self) {
        let advertisement = AdvertisementData {};
        SystemBus::new()
            .await
            .unwrap()
            .publish_object(Self::ADDRESS, advertisement)
            .await
            .unwrap();
    }

    async fn get_all_hci(&self) -> HashMap<ObjectPath<'_>, HashMap<String, HashMap<String, OwnedValue>>> {
        let proxy = BluezRootAPI::connect().await.unwrap();
        let managed_objects = proxy.get_managed_objects().await.unwrap();
        managed_objects
            .into_iter()
            .filter(|(_, services)| {
                services.contains_key("org.bluez.Adapter1") && services.contains_key("org.bluez.LEAdvertisingManager1")
            })
            .collect()
    }

    async fn get_all_hci_addresses(&self) -> Vec<String> {
        let hci = self.get_all_hci().await;
        hci.into_iter()
            .map(|(_, services)| {
                services
                    .get("org.bluez.Adapter1")
                    .unwrap()
                    .get("Address")
                    .unwrap()
                    .downcast_ref::<Str>()
                    .unwrap()
                    .to_string()
            })
            .collect()
    }

    async fn get_hci_path(&self, hci_address: &str) -> Option<ObjectPath<'_>> {
        let hci = self.get_all_hci().await;
        for (path, services) in hci {
            if services
                .get("org.bluez.Adapter1")
                .unwrap()
                .get("Address")
                .unwrap()
                .downcast_ref::<Str>()
                .unwrap()
                == hci_address
            {
                return Some(path);
            }
        }
        None
    }

    async fn enable_advertising(&self, hci_address: &str, name: String) {
        let mut active_advertisements = self.active_advertisements.lock().await;
        if active_advertisements.contains_key(hci_address) {
            self.disable_advertising(hci_address).await;
        }

        let path = self.get_hci_path(hci_address).await;
        if path.is_none() {
            panic!("Unknown hci address {}", hci_address);
        }

        let path = path.unwrap();
        let hci = BluezRootAPI::connect().await.unwrap();
        let hci = hci.get_proxy("org.bluez", &path).await.unwrap();

        if !self.pairing_manager.lock().await.enabled && active_advertisements.is_empty() {
            self.pairing_manager.lock().await.enable_automatically().await;
        }

        active_advertisements.insert(hci_address.to_string(), HciState::save(&hci));
        HciState::advertising(name).restore_on(&hci);
        hci.register_advertisement(Self::ADDRESS, HashMap::new()).await.unwrap();
    }

    async fn disable_advertising(&self, hci_address: &str) {
        let mut active_advertisements = self.active_advertisements.lock().await;
        if !active_advertisements.contains_key(hci_address) {
            panic!("No advertisement found for {}", hci_address);
        }

        let original_state = active_advertisements.remove(hci_address).unwrap();
        let path = self.get_hci_path(hci_address).await;
        if let Some(path) = path {
            let hci = BluezRootAPI::connect().await.unwrap();
            let hci = hci.get_proxy("org.bluez", &path).await.unwrap();
            hci.unregister_advertisement(Self::ADDRESS).await.unwrap();
            original_state.restore_on(&hci);
        }

        if active_advertisements.is_empty() {
            self.pairing_manager.lock().await.disable_if_enabled_automatically().await;
        }
    }
}
