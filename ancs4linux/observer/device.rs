use std::sync::{Arc, Mutex};
use log::{debug, error, info, warn};
use zbus::zvariant::ObjectPath;
use crate::common::apis::ObserverAPI;
use crate::common::external_apis::BluezGattCharacteristicAPI;
use crate::common::task_restarter::TaskRestarter;
use crate::observer::device_comm::DeviceCommunicator;

pub struct MobileDevice {
    server: Arc<Mutex<ObserverAPI>>,
    path: String,
    communicator: Option<DeviceCommunicator>,
    paired: bool,
    connected: bool,
    name: Option<String>,
    notification_source: Option<BluezGattCharacteristicAPI>,
    control_point: Option<BluezGattCharacteristicAPI>,
    data_source: Option<BluezGattCharacteristicAPI>,
}

impl MobileDevice {
    pub fn new(path: String, server: Arc<Mutex<ObserverAPI>>) -> Self {
        Self {
            server,
            path,
            communicator: None,
            paired: false,
            connected: false,
            name: None,
            notification_source: None,
            control_point: None,
            data_source: None,
        }
    }

    pub fn set_notification_source(&mut self, path: ObjectPath<'_>) {
        self.unsubscribe();
        self.notification_source = Some(BluezGattCharacteristicAPI::connect(path));
        self.try_subscribe();
    }

    pub fn set_control_point(&mut self, path: ObjectPath<'_>) {
        self.unsubscribe();
        self.control_point = Some(BluezGattCharacteristicAPI::connect(path));
        self.try_subscribe();
    }

    pub fn set_data_source(&mut self, path: ObjectPath<'_>) {
        self.unsubscribe();
        self.data_source = Some(BluezGattCharacteristicAPI::connect(path));
        self.try_subscribe();
    }

    pub fn set_paired(&mut self, paired: bool) {
        self.unsubscribe();
        self.paired = paired;
        self.try_subscribe();
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.unsubscribe();
        self.connected = connected;
        self.try_subscribe();
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
        self.try_subscribe();
    }

    fn unsubscribe(&mut self) {
        self.communicator = None;
    }

    fn try_subscribe(&mut self) {
        debug!(
            "{}: {} {} {}",
            self.path, self.paired, self.connected, self.communicator.is_none()
        );
        if !(self.paired
            && self.connected
            && self.name.is_some()
            && self.notification_source.is_some()
            && self.control_point.is_some()
            && self.data_source.is_some()
            && self.communicator.is_none())
        {
            return;
        }

        info!("Asking for notifications...");
        let path = self.path.clone();
        let server = self.server.clone();
        let notification_source = self.notification_source.clone().unwrap();
        let control_point = self.control_point.clone().unwrap();
        let data_source = self.data_source.clone().unwrap();
        TaskRestarter::new(
            120,
            1,
            move || {
                let result = Self::try_asking(
                    &path,
                    &server,
                    &notification_source,
                    &control_point,
                    &data_source,
                );
                result
            },
            || info!("Asking for notifications: success."),
            || error!("Failed to subscribe to notifications."),
        )
        .try_running_bg();
    }

    fn try_asking(
        path: &str,
        server: &Arc<Mutex<ObserverAPI>>,
        notification_source: &BluezGattCharacteristicAPI,
        control_point: &BluezGattCharacteristicAPI,
        data_source: &BluezGattCharacteristicAPI,
    ) -> bool {
        match data_source.start_notify() {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "Failed to start subscribe to notifications (is phone paired?): {}",
                    e
                );
                return false;
            }
        }
        match notification_source.start_notify() {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "Failed to start subscribe to notifications (is phone paired?): {}",
                    e
                );
                return false;
            }
        }

        let comm = DeviceCommunicator::new(Arc::new(Mutex::new(Self {
            server: server.clone(),
            path: path.to_string(),
            communicator: None,
            paired: true,
            connected: true,
            name: None,
            notification_source: Some(notification_source.clone()),
            control_point: Some(control_point.clone()),
            data_source: Some(data_source.clone()),
        })));
        comm.attach();
        let mut device = server.lock().unwrap();
        device.communicator = Some(comm);

        true
    }

    pub fn handle_action(&mut self, notification_id: u32, is_positive: bool) {
        if let Some(communicator) = &self.communicator {
            communicator.ask_for_action(notification_id, is_positive);
        }
    }
}
