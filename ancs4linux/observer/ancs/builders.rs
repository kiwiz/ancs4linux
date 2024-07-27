use std::convert::TryInto;

#[derive(Debug)]
pub struct GetNotificationAttributes {
    pub id: u32,
    pub get_positive_action: bool,
    pub get_negative_action: bool,
}

impl GetNotificationAttributes {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut msg = vec![
            CommandID::GetNotificationAttributes as u8,
            self.id.to_le_bytes()[0],
            self.id.to_le_bytes()[1],
            self.id.to_le_bytes()[2],
            self.id.to_le_bytes()[3],
            NotificationAttributeID::AppIdentifier as u8,
            NotificationAttributeID::Title as u8,
            USHORT_MAX.to_le_bytes()[0],
            USHORT_MAX.to_le_bytes()[1],
            NotificationAttributeID::Message as u8,
            USHORT_MAX.to_le_bytes()[0],
            USHORT_MAX.to_le_bytes()[1],
        ];
        if self.get_positive_action {
            msg.push(NotificationAttributeID::PositiveActionLabel as u8);
        }
        if self.get_negative_action {
            msg.push(NotificationAttributeID::NegativeActionLabel as u8);
        }
        msg
    }
}

#[derive(Debug)]
pub struct GetAppAttributes {
    pub app_id: String,
}

impl GetAppAttributes {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut msg = vec![
            CommandID::GetAppAttributes as u8,
            self.app_id.len() as u8 + 1,
        ];
        msg.extend_from_slice(self.app_id.as_bytes());
        msg.push(AppAttributeID::DisplayName as u8);
        msg
    }
}

#[derive(Debug)]
pub struct PerformNotificationAction {
    pub notification_id: u32,
    pub is_positive: bool,
}

impl PerformNotificationAction {
    pub fn to_vec(&self) -> Vec<u8> {
        vec![
            CommandID::PerformNotificationAction as u8,
            self.notification_id.to_le_bytes()[0],
            self.notification_id.to_le_bytes()[1],
            self.notification_id.to_le_bytes()[2],
            self.notification_id.to_le_bytes()[3],
            if self.is_positive {
                ActionID::Positive as u8
            } else {
                ActionID::Negative as u8
            },
        ]
    }
}
