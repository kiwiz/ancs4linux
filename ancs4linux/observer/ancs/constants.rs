pub const USHORT_MAX: u16 = u16::MAX;
pub const UINT_MAX: u32 = u32::MAX;

pub const ANCS_SERVICE: &str = "7905f431-b5ce-4e99-a40f-4b1e122d00d0";
pub const NOTIFICATION_SOURCE_CHAR: &str = "9fbf120d-6301-42d9-8c58-25e699a21dbd";
pub const CONTROL_POINT_CHAR: &str = "69d1d8f3-45e1-49a8-9821-9bbdfdaad9d9";
pub const DATA_SOURCE_CHAR: &str = "22eac6e9-24d6-4bb5-be44-b36ace7c7bfb";
pub const ANCS_CHARS: [&str; 3] = [
    NOTIFICATION_SOURCE_CHAR,
    CONTROL_POINT_CHAR,
    DATA_SOURCE_CHAR,
];

#[repr(u8)]
pub enum CategoryID {
    Other = 0,
    IncomingCall = 1,
    MissedCall = 2,
    Voicemail = 3,
    Social = 4,
    Schedule = 5,
    Email = 6,
    News = 7,
    HealthAndFitness = 8,
    BusinessAndFinance = 9,
    Location = 10,
    Entertainment = 11,
}

#[repr(u8)]
pub enum EventID {
    NotificationAdded = 0,
    NotificationModified = 1,
    NotificationRemoved = 2,
}

bitflags::bitflags! {
    pub struct EventFlag: u8 {
        const SILENT = 1 << 0;
        const IMPORTANT = 1 << 1;
        const PRE_EXISTING = 1 << 2;
        const POSITIVE_ACTION = 1 << 3;
        const NEGATIVE_ACTION = 1 << 4;
    }
}

#[repr(u8)]
pub enum CommandID {
    GetNotificationAttributes = 0,
    GetAppAttributes = 1,
    PerformNotificationAction = 2,
}

#[repr(u8)]
pub enum NotificationAttributeID {
    AppIdentifier = 0,
    Title = 1,
    Subtitle = 2,
    Message = 3,
    MessageSize = 4,
    Date = 5,
    PositiveActionLabel = 6,
    NegativeActionLabel = 7,
}

#[repr(u8)]
pub enum ActionID {
    Positive = 0,
    Negative = 1,
}

#[repr(u8)]
pub enum AppAttributeID {
    DisplayName = 0,
}
