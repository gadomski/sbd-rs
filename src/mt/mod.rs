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

/// Mobile Terminated Header
#[derive(Debug)]
struct Header {
    // IEI: 0x41 [1] (Table 5-1)
    // Header length [2]
    client_msg_id: u32, // or 4 u8?
    imei: [u8; 15],
    disposition_flags: u16, //Table 5-9
}

impl Header {
    fn len(&self) -> usize {
        21
    }

    fn write<W: std::io::Write>(&self, wtr: &mut W) -> Result<usize, Error> {
        wtr.write_u8(0x41)?;
        wtr.write_u16::<BigEndian>(21)?;
        wtr.write_u32::<BigEndian>(self.client_msg_id)?;
        wtr.write(&self.imei)?;
        wtr.write_u16::<BigEndian>(self.disposition_flags)?;
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
    use super::Header;

    #[test]
    fn header_write() {
        let header = Header {
            client_msg_id: 9999,
            imei: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
            disposition_flags: 9999,
        };
        let mut msg = vec![];
        let n = header.write(&mut msg);
        // Total size is always 24
        assert_eq!(n.unwrap(), 24);
        assert_eq!(
            msg,
            [
                0x41, 0x00, 0x15, 0x00, 0x00, 0x27, 0x0f, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x27, 0x0f
            ]
        );
    }

    #[test]
    fn header_to_vec() {
        let header = Header {
            client_msg_id: 9999,
            imei: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
            disposition_flags: 9999,
        };
        let output = header.to_vec();

        assert_eq!(
            output,
            [
                0x41, 0x00, 0x15, 0x00, 0x00, 0x27, 0x0f, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x27, 0x0f
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
enum InformationElement {
    H(Header),
    P(Payload),
}

impl InformationElement {
    fn write<W: std::io::Write>(&self, wtr: &mut W) -> Result<usize, Error> {
        match self {
            InformationElement::H(element) => element.write(wtr),
            InformationElement::P(element) => element.write(wtr),
        }
    }
}
