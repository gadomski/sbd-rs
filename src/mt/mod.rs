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

mod header;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use self::header::Header;
use crate::Error;

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
        wtr.write_all(&self.payload)?;
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
        wtr.write_all(&self.imei)?;
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
    fn len(&self) -> usize {
        match self {
            InformationElement::H(element) => element.len(),
            InformationElement::P(element) => element.len(),
            InformationElement::C(element) => element.len(),
        }
    }

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
        // Protocol version
        wtr.write_u8(1)?;
        let n: usize = self.elements.iter().map(|e| e.len()).sum();
        wtr.write_u16::<BigEndian>(
            n.try_into()
                .expect("Sum of MT information elements lengths is longer than u16"),
        )?;

        let mut n = 3;
        for e in &self.elements {
            n += e.write(wtr)?;
        }
        Ok(n)
    }
}

//size, read, write, to_vec, ...
