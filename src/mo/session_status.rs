use Result;

/// The status of a mobile-originated session.
///
/// The descriptions for these codes are taken directly from the `DirectIP` documentation.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum SessionStatus {
    /// The SBD session completed successfully.
    Ok = 0,
    /// The MO message transfer, if any, was successful. The MT message queued at the GSS is too
    /// large to be transferred within a single SBD session.
    OkMobileTerminatedTooLarge = 1,
    /// The MO message transfer, if any, was successful. The reported location was determined to be
    /// of unacceptable quality. This value is only applicable to IMEIs using SBD protocol revision
    /// 1.
    OkLocationUnacceptableQuality = 2,
    /// The SBD session timed out before session completion.
    Timeout = 10,
    /// The MO message being transferred by the IMEI is too large to be transerred within a single
    /// SBD session.
    MobileOriginatedTooLarge = 12,
    /// An RF link loss ocurred during the SBD session.
    RFLinkLoss = 13,
    /// An IMEI protocol anomaly occurred during SBD session.
    IMEIProtocolAnomaly = 14,
    /// The IMEI is prohibited from accessing the GSS.
    Prohibited = 15,
}

impl SessionStatus {
    /// Creates a new session status from a code.
    ///
    /// Returns an error if the code is unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mo::SessionStatus;
    /// assert!(SessionStatus::new(0).is_ok());
    /// assert!(SessionStatus::new(3).is_err());
    /// ```
    pub fn new(n: u8) -> Result<SessionStatus> {
        use Error;
        match n {
            0 => Ok(SessionStatus::Ok),
            1 => Ok(SessionStatus::OkMobileTerminatedTooLarge),
            2 => Ok(SessionStatus::OkLocationUnacceptableQuality),
            10 => Ok(SessionStatus::Timeout),
            12 => Ok(SessionStatus::MobileOriginatedTooLarge),
            13 => Ok(SessionStatus::RFLinkLoss),
            14 => Ok(SessionStatus::IMEIProtocolAnomaly),
            15 => Ok(SessionStatus::Prohibited),
            _ => Err(Error::UnknownSessionStatus(n)),
        }
    }
}
