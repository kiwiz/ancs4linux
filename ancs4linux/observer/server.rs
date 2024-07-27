use zbus::dbus_interface;
use zvariant::ObjectPath;
use crate::common::apis::ObserverAPI;
use crate::observer::scanner::Scanner;

pub struct ObserverServer {
    scanner: Option<Scanner>,
}

impl ObserverServer {
    pub fn new() -> Self {
        Self { scanner: None }
    }

    pub fn set_scanner(&mut self, scanner: Scanner) {
        self.scanner = Some(scanner);
    }
}

#[dbus_interface(name = "ancs4linux.Observer")]
impl ObserverAPI for ObserverServer {
    fn invoke_device_action(
        &self,
        device_handle: ObjectPath<'_>,
        notification_id: u32,
        is_positive: bool,
    ) {
        if let Some(scanner) = &self.scanner {
            if let Some(device) = scanner.devices.get(device_handle.as_str()) {
                let mut device = device.lock().unwrap();
                device.handle_action(notification_id, is_positive);
            }
        }
    }

    #[dbus_interface(signal)]
    fn show_notification(&self, json: String);

    #[dbus_interface(signal)]
    fn dismiss_notification(&self, id: u32);
}
