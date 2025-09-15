//! An information element is a portion of a SBD message.
//!
//! Information elements come after the SBD header. They come in many types,
//! including more header-type information and the actual data payload.

use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::{TimeZone, Utc};

use crate::{
    mo::{
        location::{MoLocation, MoLocationError},
        Header, SessionStatus,
    },
    Error,
};

/// A mobile-originated information element, or IE.
///
/// These are the building blocks of a SBD message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InformationElement {
    /// Information element holding the mobile-originated header.
    Header(Header),
    /// The mobile originated payload.
    Payload(Vec<u8>),
    /// The mobile originated location information.
    LocationInformation([u8; 11]),
}

impl InformationElement {
    /// Reads this information element from a `Read`.
    pub fn read_from<R: Read>(mut read: R) -> Result<InformationElement, Error> {
        let iei = read.read_u8()?;
        let length = read.read_u16::<BigEndian>()?;
        match iei {
            1 => {
                let auto_id = read.read_u32::<BigEndian>()?;
                let mut imei = [0; 15];
                read.read_exact(&mut imei)?;
                let session_status = SessionStatus::new(read.read_u8()?)?;
                let momsn = read.read_u16::<BigEndian>()?;
                let mtmsn = read.read_u16::<BigEndian>()?;
                let time_of_session =
                    read.read_u32::<BigEndian>()
                        .map_err(Error::from)
                        .and_then(|n| {
                            Utc.timestamp_opt(i64::from(n), 0)
                                .single()
                                .ok_or(Error::InvalidTimeOfSession)
                        })?;
                Ok(InformationElement::Header(Header {
                    auto_id,
                    imei,
                    session_status,
                    momsn,
                    mtmsn,
                    time_of_session,
                }))
            }
            2 => {
                let mut payload = vec![0; length as usize];
                read.read_exact(&mut payload)?;
                Ok(InformationElement::Payload(payload))
            }
            3 => {
                let mut bytes = [0; 11];
                read.read_exact(&mut bytes)?;
                Ok(InformationElement::LocationInformation(bytes))
            }
            5 => unimplemented!(),
            _ => Err(Error::InvalidInformationElementIdentifier(iei)),
        }
    }

    /// Returns the length of this information element, including the information element header.
    pub fn len(&self) -> usize {
        match *self {
            InformationElement::Header(_) => 31,
            InformationElement::Payload(ref payload) => 3 + payload.len(),
            InformationElement::LocationInformation(_) => 14,
        }
    }

    /// Returns true if this information element is empty.
    ///
    /// At this point, only can be true if the payload is empty.
    pub fn is_empty(&self) -> bool {
        match *self {
            InformationElement::Payload(ref payload) => payload.is_empty(),
            _ => false,
        }
    }

    /// Writes this information element to a `Write`.
    pub fn write_to<W: Write>(&self, mut write: W) -> Result<(), Error> {
        match *self {
            InformationElement::Header(ref header) => {
                write.write_u8(1)?;
                write.write_u16::<BigEndian>(31)?; // 28?
                write.write_u32::<BigEndian>(header.auto_id)?;
                write.write_all(&header.imei)?;
                write.write_u8(header.session_status as u8)?;
                write.write_u16::<BigEndian>(header.momsn)?;
                write.write_u16::<BigEndian>(header.mtmsn)?;
                let timestamp = header.time_of_session.timestamp();
                if timestamp < 0 {
                    return Err(Error::NegativeTimestamp(timestamp));
                } else {
                    write.write_u32::<BigEndian>(timestamp as u32)?;
                };
            }
            InformationElement::Payload(ref payload) => {
                write.write_u8(2)?;
                let len = payload.len();
                if len > u16::MAX as usize {
                    return Err(Error::PayloadTooLong(len));
                } else {
                    write.write_u16::<BigEndian>(len as u16)?;
                }
                write.write_all(payload)?;
            }
            InformationElement::LocationInformation(ref bytes) => {
                write.write_u8(3)?;
                write.write_u16::<BigEndian>(20)?; //11?
                write.write_all(bytes)?;
            }
        }
        Ok(())
    }
    /// If this is a location information element, parse it into a `MoLocation`.
    pub fn as_mo_location(&self) -> Option<Result<MoLocation, MoLocationError>> {
        match self {
            InformationElement::LocationInformation(bytes) => Some(MoLocation::parse(*bytes)),
            _ => None,
        }
    }
}

impl From<Header> for InformationElement {
    fn from(header: Header) -> InformationElement {
        InformationElement::Header(header)
    }
}

impl From<Vec<u8>> for InformationElement {
    fn from(payload: Vec<u8>) -> InformationElement {
        InformationElement::Payload(payload)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Cursor, Read, Seek, SeekFrom},
    };

    use chrono::TimeZone;

    use super::*;

    #[test]
    fn read_from() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3)).unwrap();
        {
            let read = Read::by_ref(&mut file).take(31);
            match InformationElement::read_from(read).unwrap() {
                InformationElement::Header(header) => {
                    assert_eq!(1894516585, header.auto_id);
                    assert_eq!(b"300234063904190", &header.imei);
                    assert_eq!(SessionStatus::Ok, header.session_status);
                    assert_eq!(75, header.momsn);
                    assert_eq!(0, header.mtmsn);
                    assert_eq!(
                        Utc.with_ymd_and_hms(2015, 7, 9, 18, 15, 8)
                            .single()
                            .unwrap(),
                        header.time_of_session
                    );
                }
                _ => panic!("Unexpected information element"),
            }
        }
        match InformationElement::read_from(file).unwrap() {
            InformationElement::Payload(data) => {
                assert_eq!(b"test message from pete", data.as_slice())
            }
            _ => panic!("Unexpected information element"),
        }
    }

    #[test]
    fn undersized() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3)).unwrap();
        let read = file.take(30);
        assert!(InformationElement::read_from(read).is_err());
    }

    #[test]
    fn header_len() {
        let header = Header {
            auto_id: 1,
            imei: [0; 15],
            session_status: SessionStatus::Ok,
            momsn: 1,
            mtmsn: 1,
            time_of_session: Utc
                .with_ymd_and_hms(2017, 10, 17, 12, 0, 0)
                .single()
                .unwrap(),
        };
        let ie = InformationElement::from(header);
        assert_eq!(31, ie.len());
    }

    #[test]
    fn payload_len() {
        assert_eq!(4, InformationElement::from(vec![1]).len());
    }

    #[test]
    fn location_information_len() {
        assert_eq!(10, InformationElement::LocationInformation([0; 11]).len());
    }

    #[test]
    fn roundtrip_header() {
        let header = Header {
            auto_id: 1,
            imei: [0; 15],
            session_status: SessionStatus::Ok,
            momsn: 1,
            mtmsn: 1,
            time_of_session: Utc
                .with_ymd_and_hms(2017, 10, 17, 12, 0, 0)
                .single()
                .unwrap(),
        };
        let ie = InformationElement::from(header);
        let mut cursor = Cursor::new(Vec::new());
        ie.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        assert_eq!(ie, InformationElement::read_from(cursor).unwrap());
    }

    #[test]
    fn header_time_of_session_too_old() {
        let header = Header {
            auto_id: 1,
            imei: [0; 15],
            session_status: SessionStatus::Ok,
            momsn: 1,
            mtmsn: 1,
            time_of_session: Utc
                .with_ymd_and_hms(1969, 12, 31, 23, 59, 59)
                .single()
                .unwrap(),
        };
        assert!(InformationElement::from(header)
            .write_to(Cursor::new(Vec::new()))
            .is_err());
    }

    #[test]
    fn roundtrip_payload() {
        let payload = vec![1];
        let ie = InformationElement::from(payload);
        let mut cursor = Cursor::new(Vec::new());
        ie.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        assert_eq!(ie, InformationElement::read_from(cursor).unwrap());
    }

    #[test]
    fn payload_too_long() {
        let payload = vec![0; u16::MAX as usize + 1];
        assert!(InformationElement::from(payload)
            .write_to(Cursor::new(Vec::new()))
            .is_err());
    }

    #[test]
    fn roundtrip_location_information() {
        let ie = InformationElement::LocationInformation([1; 11]);
        let mut cursor = Cursor::new(Vec::new());
        ie.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        assert_eq!(ie, InformationElement::read_from(cursor).unwrap());
    }
}
