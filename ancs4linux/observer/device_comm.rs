use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use rand::Rng;
use zbus::zvariant::Value;
use crate::common::apis::ShowNotificationData;
use crate::common::external_apis::BluezGattCharacteristicAPI;
use crate::observer::ancs::builders::{GetAppAttributes, GetNotificationAttributes, PerformNotificationAction};
use crate::observer::ancs::constants::{CommandID, EventID, UINT_MAX};
use crate::observer::ancs::parsers::{AppAttributes, DataSourceEvent, Notification, NotificationAttributes};

pub struct DeviceCommunicator {
    device: Arc<Mutex<MobileDevice>>,
    id: u32,
    notification_queue: Vec<ShowNotificationData>,
    awaiting_app_names: HashSet<String>,
    known_app_names: HashMap<String, String>,
}

impl DeviceCommunicator {
    pub fn new(device: Arc<Mutex<MobileDevice>>) -> Self {
        let id = rand::thread_rng().gen_range(1..100_000) * 1000;
        Self {
            device,
            id,
            notification_queue: Vec::new(),
            awaiting_app_names: HashSet::new(),
            known_app_names: HashMap::new(),
        }
    }

    pub async fn attach(&self) {
        let device = self.device.lock().unwrap();
        if let Some(notification_source) = &device.notification_source {
            notification_source.disconnect_properties_changed().await.unwrap();
            notification_source.connect_properties_changed(Self::on_ns_change).await.unwrap();
        }
        if let Some(data_source) = &device.data_source {
            data_source.disconnect_properties_changed().await.unwrap();
            data_source.connect_properties_changed(Self::on_ds_change).await.unwrap();
        }
    }

    async fn on_ns_change(
        &self,
        interface: &str,
        changes: &HashMap<String, Value>,
        invalidated: &[String],
    ) {
        if interface != "org.bluez.GattCharacteristic1" || !changes.contains_key("Value") {
            return;
        }

        let notification = Notification::parse(&changes["Value"].to_bytes().unwrap());
        if notification.type == EventID::NotificationAdded && notification.is_fresh() {
            self.ask_for_notification_details(notification).await;
        } else if notification.type == EventID::NotificationModified {
            self.ask_for_notification_details(notification).await;
        } else {
            let device = self.device.lock().unwrap();
            device.server.emit_dismiss_notification(notification.id).await.unwrap();
        }
    }

    async fn ask_for_notification_details(&self, notification: Notification) {
        let msg = GetNotificationAttributes {
            id: notification.id,
            get_positive_action: notification.has_positive_action(),
            get_negative_action: notification.has_negative_action(),
        };
        let device = self.device.lock().unwrap();
        if let Some(control_point) = &device.control_point {
            control_point.write_value(msg.to_list(), HashMap::new()).await.unwrap();
        }
    }

    async fn on_ds_change(
        &self,
        interface: &str,
        changes: &HashMap<String, Value>,
        invalidated: &[String],
    ) {
        if interface != "org.bluez.GattCharacteristic1" || !changes.contains_key("Value") {
            return;
        }

        let ev = DataSourceEvent::parse(&changes["Value"].to_bytes().unwrap());
        if ev.type == CommandID::GetNotificationAttributes {
            self.on_notification_attributes(ev.as_notification_attributes()).await;
        } else if ev.type == CommandID::GetAppAttributes {
            self.on_app_attributes(ev.as_app_attributes()).await;
        }
    }

    async fn on_notification_attributes(&self, attrs: NotificationAttributes) {
        let device = self.device.lock().unwrap();
        let data = ShowNotificationData {
            device_handle: device.path.clone(),
            device_name: device.name.clone().unwrap_or_default(),
            app_id: attrs.app_id.clone(),
            app_name: String::new(),
            id: (self.id + attrs.id) % UINT_MAX,
            title: attrs.title.clone(),
            body: attrs.message.clone(),
            positive_action: attrs.positive_action.clone(),
            negative_action: attrs.negative_action.clone(),
        };
        self.queue_notification(data).await;
        self.process_queue().await;
    }

    async fn on_app_attributes(&self, attrs: AppAttributes) {
        self.known_app_names.insert(attrs.app_id.clone(), attrs.app_name.clone());
        self.awaiting_app_names.remove(&attrs.app_id);
        self.process_queue().await;
    }

    async fn queue_notification(&self, data: ShowNotificationData) {
        self.notification_queue.push(data);
    }

    async fn ask_for_app_name(&self, app_id: &str) {
        self.awaiting_app_names.insert(app_id.to_string());
        let msg = GetAppAttributes {
            app_id: app_id.to_string(),
        };
        let device = self.device.lock().unwrap();
        if let Some(control_point) = &device.control_point {
            control_point.write_value(msg.to_list(), HashMap::new()).await.unwrap();
        }
    }

    async fn process_queue(&self) {
        let mut unprocessed = Vec::new();
        for data in &self.notification_queue {
            if !data.app_name.is_empty() {
                let device = self.device.lock().unwrap();
                device.server.emit_show_notification(data.clone()).await.unwrap();
            } else if let Some(app_name) = self.known_app_names.get(&data.app_id) {
                let mut data = data.clone();
                data.app_name = app_name.clone();
                let device = self.device.lock().unwrap();
                device.server.emit_show_notification(data).await.unwrap();
            } else if self.awaiting_app_names.contains(&data.app_id) {
                unprocessed.push(data.clone());
            } else {
                self.ask_for_app_name(&data.app_id).await;
                unprocessed.push(data.clone());
            }
        }
        self.notification_queue = unprocessed;
    }

    async fn ask_for_action(&self, notification_id: u32, is_positive: bool) {
        let id = (notification_id - self.id) % UINT_MAX;
        let msg = PerformNotificationAction {
            notification_id: id,
            is_positive,
        };
        let device = self.device.lock().unwrap();
        if let Some(control_point) = &device.control_point {
            control_point.write_value(msg.to_list(), HashMap::new()).await.unwrap();
        }
    }
}
