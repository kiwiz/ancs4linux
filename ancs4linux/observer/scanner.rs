use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zbus::zvariant::{ObjectPath, OwnedValue, Value};
use zbus::Connection;
use crate::common::apis::ObserverAPI;
use crate::common::external_apis::{BluezDeviceAPI, BluezGattCharacteristicAPI, BluezRootAPI};
use crate::observer::ancs::constants::{ANCS_CHARS, CONTROL_POINT_CHAR, DATA_SOURCE_CHAR, NOTIFICATION_SOURCE_CHAR};
use crate::observer::device::MobileDevice;

pub struct Scanner {
    server: Arc<Mutex<ObserverAPI>>,
    root: BluezRootAPI,
    devices: HashMap<String, Arc<Mutex<MobileDevice>>>,
    property_observers: HashMap<String, BluezDeviceAPI>,
}

impl Scanner {
    pub fn new(server: Arc<Mutex<ObserverAPI>>) -> Self {
        let root = BluezRootAPI::connect().unwrap();
        Self {
            server,
            root,
            devices: HashMap::new(),
            property_observers: HashMap::new(),
        }
    }

    pub async fn start_observing(&mut self) {
        let root = self.root.clone();
        let server = self.server.clone();
        root.connect_interfaces_added(move |path, services| {
            let mut scanner = Scanner {
                server: server.clone(),
                root: root.clone(),
                devices: HashMap::new(),
                property_observers: HashMap::new(),
            };
            scanner.process_object(path, services);
        }).await.unwrap();

        root.connect_interfaces_removed(move |path, services| {
            let mut scanner = Scanner {
                server: server.clone(),
                root: root.clone(),
                devices: HashMap::new(),
                property_observers: HashMap::new(),
            };
            scanner.remove_observers(path, services);
        }).await.unwrap();

        let objects = root.get_managed_objects().await.unwrap();
        for (path, services) in objects {
            self.process_object(path, services).await;
        }
    }

    async fn process_object(&mut self, path: ObjectPath<'_>, services: HashMap<String, OwnedValue>) {
        if services.contains_key(BluezDeviceAPI::INTERFACE) {
            if !self.property_observers.contains_key(path.as_str()) {
                let device_api = BluezDeviceAPI::connect(path.clone()).await.unwrap();
                self.property_observers.insert(path.to_string(), device_api.clone());
                device_api.connect_properties_changed(move |interface, changes, invalidated| {
                    let mut scanner = Scanner {
                        server: server.clone(),
                        root: root.clone(),
                        devices: HashMap::new(),
                        property_observers: HashMap::new(),
                    };
                    scanner.process_property(path.clone(), interface, changes, invalidated);
                }).await.unwrap();
                self.process_property(path.clone(), BluezDeviceAPI::INTERFACE, services[BluezDeviceAPI::INTERFACE].clone(), vec![]).await;
            }
        }

        if services.contains_key(BluezGattCharacteristicAPI::INTERFACE) {
            let uuid = services[BluezGattCharacteristicAPI::INTERFACE]["UUID"].to_str().unwrap();
            if ANCS_CHARS.contains(&uuid) {
                let device_path = path.as_str().rsplitn(2, '/').nth(1).unwrap().to_string();
                let device = self.devices.entry(device_path.clone()).or_insert_with(|| {
                    Arc::new(Mutex::new(MobileDevice::new(device_path.clone(), self.server.clone())))
                });
                let mut device = device.lock().unwrap();
                if uuid == NOTIFICATION_SOURCE_CHAR {
                    device.set_notification_source(path.clone());
                } else if uuid == CONTROL_POINT_CHAR {
                    device.set_control_point(path.clone());
                } else if uuid == DATA_SOURCE_CHAR {
                    device.set_data_source(path.clone());
                }
            }
        }
    }

    async fn process_property(&mut self, device: ObjectPath<'_>, interface: &str, changes: HashMap<String, OwnedValue>, invalidated: Vec<String>) {
        if interface == BluezDeviceAPI::INTERFACE {
            let device = self.devices.entry(device.to_string()).or_insert_with(|| {
                Arc::new(Mutex::new(MobileDevice::new(device.to_string(), self.server.clone())))
            });
            let mut device = device.lock().unwrap();
            if let Some(paired) = changes.get("Paired") {
                device.set_paired(paired.to_bool().unwrap());
            }
            if let Some(connected) = changes.get("Connected") {
                device.set_connected(connected.to_bool().unwrap());
            }
            if let Some(alias) = changes.get("Alias") {
                device.set_name(alias.to_str().unwrap().to_string());
            }
        }
    }

    async fn remove_observers(&mut self, path: ObjectPath<'_>, services: Vec<String>) {
        if let Some(device_api) = self.property_observers.remove(path.as_str()) {
            device_api.disconnect_properties_changed().await.unwrap();
        }
    }
}
