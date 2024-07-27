use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::dbus_interface;
use zvariant::ObjectPath;

use crate::common::apis::{AdvertisingAPI, PairingAgentAPI};
use crate::common::dbus::{PairingRejected, Str, SystemBus, UInt16, UInt32};
use crate::common::external_apis::BluezAgentManagerAPI;

#[dbus_interface(name = "org.bluez.Agent1")]
struct PairingAgent {
    server: Arc<Mutex<dyn AdvertisingAPI>>,
}

impl PairingAgent {
    fn new(server: Arc<Mutex<dyn AdvertisingAPI>>) -> Self {
        Self { server }
    }

    fn release(&self) {}

    fn request_pin_code(&self, _device: ObjectPath<'_>) -> Result<Str, PairingRejected> {
        Err(PairingRejected)
    }

    fn display_pin_code(&self, _device: ObjectPath<'_>, _pincode: Str) -> Result<(), PairingRejected> {
        Err(PairingRejected)
    }

    fn request_passkey(&self, _device: ObjectPath<'_>) -> Result<UInt32, PairingRejected> {
        Err(PairingRejected)
    }

    fn display_passkey(&self, _device: ObjectPath<'_>, _passkey: UInt32, _entered: UInt16) -> Result<(), PairingRejected> {
        Err(PairingRejected)
    }

    fn request_confirmation(&self, _device: ObjectPath<'_>, passkey: UInt32) -> Result<(), PairingRejected> {
        let server = self.server.clone();
        tokio::spawn(async move {
            server.lock().await.emit_pairing_code(&passkey.to_string());
        });
        Ok(())
    }

    fn request_authorization(&self, _device: ObjectPath<'_>) -> Result<(), PairingRejected> {
        Err(PairingRejected)
    }

    fn authorize_service(&self, _device: ObjectPath<'_>, _uuid: Str) -> Result<(), PairingRejected> {
        Err(PairingRejected)
    }

    fn cancel(&self) {}
}

struct PairingManager {
    enabled: bool,
    enabled_automatically: bool,
    agent_manager: BluezAgentManagerAPI,
}

impl PairingManager {
    fn new() -> Self {
        Self {
            enabled: false,
            enabled_automatically: false,
            agent_manager: BluezAgentManagerAPI::connect().unwrap(),
        }
    }

    async fn register(&self, server: Arc<Mutex<dyn AdvertisingAPI>>) {
        let agent = PairingAgent::new(server);
        SystemBus::new()
            .await
            .unwrap()
            .publish_object(PairingAgentAPI::path(), agent)
            .await
            .unwrap();
    }

    async fn enable(&mut self) {
        if self.enabled {
            return;
        }

        self.agent_manager
            .register_agent(PairingAgentAPI::path(), "DisplayYesNo")
            .await
            .unwrap();
        self.agent_manager
            .request_default_agent(PairingAgentAPI::path())
            .await
            .unwrap();
        self.enabled = true;
        self.enabled_automatically = false;
    }

    async fn disable(&mut self) {
        if !self.enabled {
            return;
        }

        self.agent_manager
            .unregister_agent(PairingAgentAPI::path())
            .await
            .unwrap();
        self.enabled = false;
        self.enabled_automatically = false;
    }

    async fn enable_automatically(&mut self) {
        if self.enabled {
            return;
        }

        self.enable().await;
        self.enabled_automatically = true;
    }

    async fn disable_if_enabled_automatically(&mut self) {
        if !self.enabled || !self.enabled_automatically {
            return;
        }

        self.disable().await;
    }
}
