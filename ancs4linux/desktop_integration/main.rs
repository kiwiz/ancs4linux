use structopt::StructOpt;
use tokio::runtime::Runtime;
use log::{info, debug};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::dbus_interface;
use zbus::zvariant::ObjectPath;

use crate::common::apis::{AdvertisingAPI, ObserverAPI, ShowNotificationData};
use crate::common::dbus::{SystemBus, UInt32, Int32};
use crate::common::external_apis::NotificationAPI;

#[derive(StructOpt, Debug)]
#[structopt(name = "ancs4linux")]
struct Opt {
    #[structopt(long, default_value = "ancs4linux.Observer", help = "Observer service path")]
    observer_dbus: String,
    #[structopt(long, default_value = "ancs4linux.Advertising", help = "Advertising service path")]
    advertising_dbus: String,
    #[structopt(long, default_value = "5000", help = "How long to show notifications for")]
    notification_ms: i32,
}

struct Notification {
    device_id: u32,
    device_handle: Option<String>,
    host_id: u32,
}

impl Notification {
    fn new(id: u32) -> Self {
        Self {
            device_id: id,
            device_handle: None,
            host_id: 0,
        }
    }

    fn show(&mut self, data: ShowNotificationData, notification_api: &NotificationAPI, notification_timeout: i32) {
        self.device_handle = Some(data.device_handle);
        let mut actions = vec![];
        if let Some(positive_action) = data.positive_action {
            actions.push("positive-action".to_string());
            actions.push(positive_action);
        }
        if let Some(negative_action) = data.negative_action {
            actions.push("negative-action".to_string());
            actions.push(negative_action);
        }
        self.host_id = notification_api.notify(
            &format!("{} ({})", data.app_name, data.device_name),
            UInt32(self.host_id),
            "",
            &data.title,
            &data.body,
            actions,
            vec![],
            Int32(notification_timeout),
        );
        debug!("Shown {} from {}.", self.host_id, data.app_name);
    }

    fn dismiss(&mut self, notification_api: &NotificationAPI) {
        if self.host_id != 0 {
            notification_api.close_notification(UInt32(self.host_id));
            debug!("Hidden {}.", self.host_id);
            self.host_id = 0;
        }
    }

    fn on_action(&self, action: &str, observer_api: &ObserverAPI) {
        if let Some(device_handle) = &self.device_handle {
            if action == "positive-action" {
                observer_api.invoke_device_action(device_handle, UInt32(self.device_id), true);
            } else if action == "negative-action" {
                observer_api.invoke_device_action(device_handle, UInt32(self.device_id), false);
            }
        }
    }
}

async fn pairing_code(pin: String, notification_api: &NotificationAPI) {
    notification_api.notify(
        "ancs4linux",
        UInt32(0),
        "",
        "Pairing initiated",
        &format!("Pair if PIN is {}", pin),
        vec![],
        vec![],
        Int32(30000),
    );
}

async fn new_notification(json: String, notifications: Arc<Mutex<HashMap<u32, Notification>>>, notification_api: &NotificationAPI) {
    let data = ShowNotificationData::parse(&json);
    let mut notifications = notifications.lock().await;
    let notification = notifications.entry(data.id).or_insert_with(|| Notification::new(data.id));
    notification.show(data, notification_api, 5000);
}

async fn dismiss_notification(id: u32, notifications: Arc<Mutex<HashMap<u32, Notification>>>, notification_api: &NotificationAPI) {
    let mut notifications = notifications.lock().await;
    if let Some(notification) = notifications.get_mut(&id) {
        notification.dismiss(notification_api);
    }
}

async fn action_clicked(host_id: u32, action: String, notifications: Arc<Mutex<HashMap<u32, Notification>>>, observer_api: &ObserverAPI) {
    let notifications = notifications.lock().await;
    for notification in notifications.values() {
        if notification.host_id == host_id {
            notification.on_action(&action, observer_api);
        }
    }
}

async fn notification_closed(id: u32, notifications: Arc<Mutex<HashMap<u32, Notification>>>) {
    let mut notifications = notifications.lock().await;
    notifications.remove(&id);
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    env_logger::init();

    let notification_timeout = opt.notification_ms;
    let notifications = Arc::new(Mutex::new(HashMap::new()));

    let notification_api = NotificationAPI::connect().await;
    let advertising_api = AdvertisingAPI::connect(&opt.advertising_dbus).await;
    let observer_api = ObserverAPI::connect(&opt.observer_dbus).await;

    notification_api.action_invoked(action_clicked);
    notification_api.notification_closed(notification_closed);
    advertising_api.pairing_code(pairing_code);
    observer_api.show_notification(new_notification);
    observer_api.dismiss_notification(dismiss_notification);

    info!("Listening to notifications...");
    SystemBus::new().await.unwrap().run().await.unwrap();
}
