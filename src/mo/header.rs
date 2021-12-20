use crate::mo::SessionStatus;
use chrono::{DateTime, Utc};

/// A mobile-originated header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    /// The Iridium Gateway id for this message.
    pub auto_id: u32,
    /// The device id.
    pub imei: [u8; 15],
    /// The session status.
    pub session_status: SessionStatus,
    /// The mobile originated message sequence number.
    pub momsn: u16,
    /// The mobile terminated message sequence number.
    pub mtmsn: u16,
    /// The time of iridium session.
    pub time_of_session: DateTime<Utc>,
}
