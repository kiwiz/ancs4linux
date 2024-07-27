use std::convert::TryInto;
use std::str;

use crate::ancs::constants::{
    CommandID, EventFlag, EventID, NotificationAttributeID,
};

#[derive(Debug)]
pub struct Notification {
    pub id: u32,
    pub event_id: EventID,
    pub event_flags: EventFlag,
}

impl Notification {
    pub fn parse(data: &[u8]) -> Self {
        let id = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let event_id = match data[0] {
            0 => EventID::NotificationAdded,
            1 => EventID::NotificationModified,
            2 => EventID::NotificationRemoved,
            _ => panic!("Invalid EventID"),
        };
        let event_flags = EventFlag::from_bits(data[1]).unwrap();
        Notification {
            id,
            event_id,
            event_flags,
        }
    }

    pub fn is_preexisting(&self) -> bool {
        self.event_flags.contains(EventFlag::PRE_EXISTING)
    }

    pub fn is_fresh(&self) -> bool {
        !self.is_preexisting()
    }

    pub fn has_positive_action(&self) -> bool {
        self.event_flags.contains(EventFlag::POSITIVE_ACTION)
    }

    pub fn has_negative_action(&self) -> bool {
        self.event_flags.contains(EventFlag::NEGATIVE_ACTION)
    }
}

#[derive(Debug)]
pub struct DataSourceEvent {
    pub command_id: CommandID,
    pub body: Vec<u8>,
}

impl DataSourceEvent {
    pub fn parse(data: &[u8]) -> Self {
        let command_id = match data[0] {
            0 => CommandID::GetNotificationAttributes,
            1 => CommandID::GetAppAttributes,
            2 => CommandID::PerformNotificationAction,
            _ => panic!("Invalid CommandID"),
        };
        let body = data[1..].to_vec();
        DataSourceEvent { command_id, body }
    }

    pub fn as_notification_attributes(&self) -> NotificationAttributes {
        assert_eq!(self.command_id, CommandID::GetNotificationAttributes);
        NotificationAttributes::parse(&self.body)
    }

    pub fn as_app_attributes(&self) -> AppAttributes {
        assert_eq!(self.command_id, CommandID::GetAppAttributes);
        AppAttributes::parse(&self.body)
    }
}

#[derive(Debug)]
pub struct NotificationAttributes {
    pub id: u32,
    pub app_id: String,
    pub title: String,
    pub message: String,
    pub positive_action: Option<String>,
    pub negative_action: Option<String>,
}

impl NotificationAttributes {
    pub fn parse(data: &[u8]) -> Self {
        let id = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let (app_id, data) = parse_string(&data[4..]);
        let (title, data) = parse_string(&data);
        let (message, mut data) = parse_string(&data);
        let mut positive_action = None;
        let mut negative_action = None;
        if !data.is_empty() && data[0] == NotificationAttributeID::PositiveActionLabel as u8 {
            let (action, remaining_data) = parse_string(&data[1..]);
            positive_action = Some(action);
            data = remaining_data;
        }
        if !data.is_empty() && data[0] == NotificationAttributeID::NegativeActionLabel as u8 {
            let (action, remaining_data) = parse_string(&data[1..]);
            negative_action = Some(action);
            data = remaining_data;
        }
        NotificationAttributes {
            id,
            app_id,
            title,
            message,
            positive_action,
            negative_action,
        }
    }
}

#[derive(Debug)]
pub struct AppAttributes {
    pub app_id: String,
    pub app_name: String,
}

impl AppAttributes {
    pub fn parse(data: &[u8]) -> Self {
        let (app_id, data) = parse_string(data);
        let app_name = if data.is_empty() {
            "<not installed>".to_string()
        } else {
            let (name, _) = parse_string(data);
            name
        };
        AppAttributes { app_id, app_name }
    }
}

fn parse_string(data: &[u8]) -> (String, &[u8]) {
    let len = u16::from_le_bytes(data[0..2].try_into().unwrap()) as usize;
    let string = str::from_utf8(&data[2..2 + len]).unwrap().to_string();
    (string, &data[2 + len..])
}
