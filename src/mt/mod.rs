//! Mobile Terminated
//!
//!

/*
Information Element Identifiers:
    0x01 MO Header IEI
    0x02 MO Payload IEI
    0x03 MO Lat/Lon Location Information IEI
    0x05 MO Confirmation IEI

    0x41 MT Header IEI
    0x42 MT Payload IEI
    0x43 MT Lat/Lon Location Information IEI
    0x44 MT Confirmation Message IEI
    0x45 MT LAC/Cell ID Location Informatio IEI


Protocol Revision Number        1   1
Overall Message Length          2   97
MT Header IEI                   1   0x41
MT Header Length                2   21
Unique Client Message ID        4   “Msg1”
IMEI (User ID)                  15  300034010123450
MT Disposition Flags            2   0x0000
MT Payload IEI                  1   0x42
MT Payload Length               2   70
MT Payload                      70  Payload Bytes
*/

use byteorder::{BigEndian, WriteBytesExt};

use crate::Error;

#[derive(Debug, PartialEq)]
/// Disposition Flags
///
/// Note: byte 3 was not defined at this point, skipping to 3rd.
/// Therefore, all flags on is currently 0b0000_0000_0011_1011.
///
/// Table 5-9
struct DispositionFlags {
    flush_queue: bool,
    send_ring_alert: bool,
    update_location: bool,
    high_priority: bool,
    assign_mtmsn: bool,
}

impl DispositionFlags {
    /// Decode a u16 into a DispositionFlags
    ///
    /// Each flag is encoded by a bit in a specific position, which
    /// is on (true) or off (false).
    ///
    /// Flag values:
    /// * Flush MT Queue (1): Delete all MT payloads in the SSD’s MT
    ///   queue
    /// * Send Ring Alert - Mo MTM (2): Send ring alert with no
    ///   associated MT payload (normal ring alert rules apply)
    /// * Update SSD Location (8): Update SSD location with given
    ///   lat/lon values
    /// * High Priority Message (16): Place the associated MT payload in
    ///   front of queue
    /// * Assign MTMSN (32): Use the value in the Unique ID field as the
    ///   MTMSN
    ///
    /// # Notes:
    ///
    /// - All non used bits are ignored. It might be useful to
    ///   consider a more strict approach, where this would fail if
    ///   non-expected bit is activated.
    fn decode(code: u16) -> Self {
        let flush_queue = match code & 0b0000_0000_0000_0001 {
            0 => false,
            _ => true,
        };
        let send_ring_alert = match code & 0b0000_0000_0000_0010 {
            0 => false,
            _ => true,
        };
        let update_location = match code & 0b0000_0000_0000_1000 {
            0 => false,
            _ => true,
        };
        let high_priority = match code & 0b0000_0000_0001_0000 {
            0 => false,
            _ => true,
        };
        let assign_mtmsn = match code & 0b0000_0000_0010_0000 {
            0 => false,
            _ => true,
        };

        DispositionFlags {
            flush_queue,
            send_ring_alert,
            update_location,
            high_priority,
            assign_mtmsn,
        }
    }

    /// Parse a DispositionFlags from a Read trait
    fn read_from<R: std::io::Read>(mut read: R) -> Result<Self, Error> {
        let code = read.read_u16::<BigEndian>()?;
        Ok(DispositionFlags::decode(code))
    }

    fn encode(&self) -> u16 {
        (u16::from(self.assign_mtmsn) << 5)
            + (u16::from(self.high_priority) << 4)
            + (u16::from(self.update_location) << 3)
            + (u16::from(self.send_ring_alert) << 1)
            + u16::from(self.flush_queue)
    }

    fn write<W: std::io::Write>(&self, wtr: &mut W) -> Result<usize, Error> {
        wtr.write_u16::<BigEndian>(self.encode())?;
        Ok(2)
    }
}

#[cfg(test)]
mod test_disposition_flags {
    use super::DispositionFlags;

    #[test]
    fn decode_all_false() {
        let ans = DispositionFlags {
            flush_queue: false,
            send_ring_alert: false,
            update_location: false,
            high_priority: false,
            assign_mtmsn: false,
        };
        assert_eq!(ans, DispositionFlags::decode(0));
    }

    #[test]
    fn decode_flush_queue() {
        let ans = DispositionFlags {
            flush_queue: true,
            send_ring_alert: false,
            update_location: false,
            high_priority: false,
            assign_mtmsn: false,
        };
        assert_eq!(ans, DispositionFlags::decode(1));
    }

    #[test]
    fn decode_send_ring_alert() {
        let ans = DispositionFlags {
            flush_queue: false,
            send_ring_alert: true,
            update_location: false,
            high_priority: false,
            assign_mtmsn: false,
        };
        assert_eq!(ans, DispositionFlags::decode(2));
    }

    #[test]
    fn decode_update_location() {
        let ans = DispositionFlags {
            flush_queue: false,
            send_ring_alert: false,
            update_location: true,
            high_priority: false,
            assign_mtmsn: false,
        };
        assert_eq!(ans, DispositionFlags::decode(8));
    }

    #[test]
    fn decode_all_true() {
        let ans = DispositionFlags {
            flush_queue: true,
            send_ring_alert: true,
            update_location: true,
            high_priority: true,
            assign_mtmsn: true,
        };
        assert_eq!(ans, DispositionFlags::decode(59));
    }

    #[test]
    fn encode_all_false() {
        let flags = DispositionFlags {
            flush_queue: false,
            send_ring_alert: false,
            update_location: false,
            high_priority: false,
            assign_mtmsn: false,
        };
        assert_eq!(flags.encode(), 0);
    }

    #[test]
    fn encode_flush_queue() {
        let flags = DispositionFlags {
            flush_queue: true,
            send_ring_alert: false,
            update_location: false,
            high_priority: false,
            assign_mtmsn: false,
        };
        assert_eq!(flags.encode(), 1);
    }

    #[test]
    fn encode_send_ring_alert() {
        let flags = DispositionFlags {
            flush_queue: false,
            send_ring_alert: true,
            update_location: false,
            high_priority: false,
            assign_mtmsn: false,
        };
        assert_eq!(flags.encode(), 2);
    }

    #[test]
    fn encode_update_location() {
        let flags = DispositionFlags {
            flush_queue: false,
            send_ring_alert: false,
            update_location: true,
            high_priority: false,
            assign_mtmsn: false,
        };
        assert_eq!(flags.encode(), 8);
    }

    #[test]
    fn encode_high_priority() {
        let flags = DispositionFlags {
            flush_queue: false,
            send_ring_alert: false,
            update_location: false,
            high_priority: true,
            assign_mtmsn: false,
        };
        assert_eq!(flags.encode(), 16);
    }

    #[test]
    fn encode_assign_mtmsn() {
        let flags = DispositionFlags {
            flush_queue: false,
            send_ring_alert: false,
            update_location: false,
            high_priority: false,
            assign_mtmsn: true,
        };
        assert_eq!(flags.encode(), 32);
    }

    #[test]
    fn encode_all_true() {
        let flags = DispositionFlags {
            flush_queue: true,
            send_ring_alert: true,
            update_location: true,
            high_priority: true,
            assign_mtmsn: true,
        };
        assert_eq!(flags.encode(), 59);
    }

    #[test]
    fn roundtrip_decode_encode() {
        let combinations = vec![
            1, 2, 3, 8, 9, 10, 11, 16, 17, 18, 19, 24, 25, 26, 27, 32, 33, 34, 35, 40, 41, 42, 43,
            48, 49, 50, 51, 56, 57, 58, 59,
        ];
        for i in combinations {
            assert_eq!(i, DispositionFlags::decode(i).encode())
        }
    }
}

/// Mobile Terminated Header
#[derive(Debug)]
struct Header {
    // IEI: 0x41 [1] (Table 5-1)
    // Header length [2]
    client_msg_id: u32, // or 4 u8?
    imei: [u8; 15],
    disposition_flags: DispositionFlags,
}

impl Header {
    fn len(&self) -> usize {
        21
    }

    fn read_from<R: std::io::Read>(mut read: R) -> Result<Header, Error> {
        let client_msg_id = read.read_u32::<BigEndian>()?;
        let mut imei = [0; 15];
        read.read_exact(&mut imei)?;
        let disposition_flags = DispositionFlags::read_from(read)?;

        Ok(Header {
            client_msg_id,
            imei,
            disposition_flags,
        })
    }

    fn write<W: std::io::Write>(&self, wtr: &mut W) -> Result<usize, Error> {
        wtr.write_u8(0x41)?;
        wtr.write_u16::<BigEndian>(21)?;
        wtr.write_u32::<BigEndian>(self.client_msg_id)?;
        wtr.write(&self.imei)?;
        self.disposition_flags.write(wtr)?;
        Ok(24)
    }

    // Export header to a vec of bytes
    fn to_vec(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        self.write(&mut buffer)
            .expect("Failed to write MT-Header to a vec.");
        buffer
    }
}

#[cfg(test)]
mod test_mt_header {
    use super::{DispositionFlags, Header};

    #[test]
    fn header_write() {
        let header = Header {
            client_msg_id: 9999,
            imei: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
            disposition_flags: DispositionFlags {
                flush_queue: true,
                send_ring_alert: true,
                update_location: true,
                high_priority: true,
                assign_mtmsn: true,
            },
        };
        let mut msg = vec![];
        let n = header.write(&mut msg);
        // Total size is always 24
        assert_eq!(n.unwrap(), 24);
        assert_eq!(
            msg,
            [
                0x41, 0x00, 0x15, 0x00, 0x00, 0x27, 0x0f, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x00, 0x3b
            ]
        );
    }

    #[test]
    fn header_to_vec() {
        let header = Header {
            client_msg_id: 9999,
            imei: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
            disposition_flags: DispositionFlags {
                flush_queue: true,
                send_ring_alert: true,
                update_location: true,
                high_priority: true,
                assign_mtmsn: true,
            },
        };
        let output = header.to_vec();

        assert_eq!(
            output,
            [
                0x41, 0x00, 0x15, 0x00, 0x00, 0x27, 0x0f, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x00, 0x3b
            ]
        );
    }
}

#[derive(Debug)]
/// Mobile Terminated Payload
///
/// Note that length is a 2-bytes and valid range is 1-1890
struct Payload {
    payload: Vec<u8>,
}

impl Payload {
    fn len(&self) -> usize {
        self.payload.len()
    }

    fn write<W: std::io::Write>(&self, wtr: &mut W) -> Result<usize, Error> {
        wtr.write_u8(0x42)?;
        let n = self.payload.len();
        wtr.write_u16::<BigEndian>(
            n.try_into()
                .expect("MT Payload's length was supposed to be u16"),
        )?;
        wtr.write(&self.payload)?;
        Ok(3 + n)
    }
}

#[derive(Debug)]
struct Confirmation {
    // From Client (not MTMSN)
    client_msg_id: u32,
    // ASCII Numeric Characters
    imei: [u8; 15],
    // 0 – 4294967295
    // It will be zero when there is an error in processing the message
    id_reference: u32,
    // Order of message in SSD's queue or error reference
    message_status: i16,
}

impl Confirmation {
    fn len(&self) -> usize {
        25
    }

    fn write<W: std::io::Write>(&self, wtr: &mut W) -> Result<usize, Error> {
        wtr.write_u8(0x44)?;
        wtr.write_u16::<BigEndian>(25)?;
        wtr.write_u32::<BigEndian>(self.client_msg_id)?;
        wtr.write(&self.imei)?;
        wtr.write_u32::<BigEndian>(self.id_reference)?;
        wtr.write_i16::<BigEndian>(self.message_status)?;
        Ok(28)
    }
}

#[cfg(test)]
mod test_mt_confirmation {
    use super::Confirmation;

    #[test]
    fn confirmation_write() {
        let confirmation = Confirmation {
            client_msg_id: 9999,
            imei: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
            id_reference: 4294967295,
            message_status: -11,
        };
        let mut msg = vec![];
        let n = confirmation.write(&mut msg);
        // Total size is always 28
        assert_eq!(n.unwrap(), 28);
        assert_eq!(
            msg,
            [
                0x44, 0x00, 0x19, 0x00, 0x00, 0x27, 0x0f, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf5
            ]
        );
    }
}

#[derive(Debug)]
enum InformationElement {
    H(Header),
    P(Payload),
    C(Confirmation),
}

impl InformationElement {
    fn write<W: std::io::Write>(&self, wtr: &mut W) -> Result<usize, Error> {
        match self {
            InformationElement::H(element) => element.write(wtr),
            InformationElement::P(element) => element.write(wtr),
            InformationElement::C(element) => element.write(wtr),
        }
    }
}

#[derive(Debug)]
struct MTMessage {
    elements: Vec<InformationElement>,
}

impl MTMessage {
    fn write<W: std::io::Write>(&self, wtr: &mut W) -> Result<usize, Error> {
        let mut n = 0;
        for e in &self.elements {
            n += e.write(wtr)?;
        }
        return Ok(n);
    }
}

//size, read, write, to_vec, ...
