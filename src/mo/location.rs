#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoLocation {
    /// 0 = North, 1 = South (per spec)
    pub north: bool,
    /// 0 = East, 1 = West (per spec)
    pub east: bool,

    /// 0..=90
    pub lat_deg: u8,
    /// 0..=59_999 (thousandths of a minute)
    pub lat_thousandths_min: u16,

    /// 0..=180
    pub lon_deg: u8,
    /// 0..=59_999 (thousandths of a minute)
    pub lon_thousandths_min: u16,

    /// Circular Error Probable radius in kilometers (80% probability).
    pub cep_km: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum MoLocationError {
    ReservedBitsNonZero(u8),
    LatDegOutOfRange(u8),
    LatThousandthsOutOfRange(u16),
    LonDegOutOfRange(u8),
    LonThousandthsOutOfRange(u16),
}

impl MoLocation {
    /// Parse the 11-byte MO Location IE value into fields.
    ///
    /// Layout (11 bytes total):
    ///   b0:  NSI/EWI flags + reserved/format (reserved should be 0)
    ///   b1:  latitude degrees (0..=90)
    ///   b2:  latitude thousandths-of-minute (MSB)
    ///   b3:  latitude thousandths-of-minute (LSB)
    ///   b4:  longitude degrees (0..=180)
    ///   b5:  longitude thousandths-of-minute (MSB)
    ///   b6:  longitude thousandths-of-minute (LSB)
    ///   b7..b10: CEP radius (u32, km), big-endian
    ///
    /// NSI: 0=North, 1=South; EWI: 0=East, 1=West.
    pub fn parse(bytes: [u8; 11]) -> Result<Self, MoLocationError> {
        let b0 = bytes[0];

        // Per spec, reserved/format code bits should be 0.
        // Only the two lowest bits (we use bit0/bit1) are meaningful (NSI/EWI).
        let reserved = b0 & !0b0000_0011;
        if reserved != 0 {
            // Non-fatal spec violation in the field — up to you whether to hard error or just warn.
            return Err(MoLocationError::ReservedBitsNonZero(b0));
        }

        // Choose bit 0 for NSI and bit 1 for EWI.
        // Spec states 0=North,1=South and 0=East,1=West respectively.
        let north = (b0 & 0b0000_0001) == 0;
        let east = (b0 & 0b0000_0010) == 0;

        let lat_deg = bytes[1];
        if lat_deg > 90 {
            return Err(MoLocationError::LatDegOutOfRange(lat_deg));
        }

        let lat_thousandths_min = u16::from_be_bytes([bytes[2], bytes[3]]);
        if lat_thousandths_min > 59_999 {
            return Err(MoLocationError::LatThousandthsOutOfRange(
                lat_thousandths_min,
            ));
        }

        let lon_deg = bytes[4];
        if lon_deg > 180 {
            return Err(MoLocationError::LonDegOutOfRange(lon_deg));
        }

        let lon_thousandths_min = u16::from_be_bytes([bytes[5], bytes[6]]);
        if lon_thousandths_min > 59_999 {
            return Err(MoLocationError::LonThousandthsOutOfRange(
                lon_thousandths_min,
            ));
        }

        let cep_km = u32::from_be_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]);

        Ok(MoLocation {
            north,
            east,
            lat_deg,
            lat_thousandths_min,
            lon_deg,
            lon_thousandths_min,
            cep_km,
        })
    }

    /// Decimal degrees, positive north, negative south.
    pub fn latitude_deg(&self) -> f64 {
        let minutes = self.lat_thousandths_min as f64 / 1000.0;
        let dd = self.lat_deg as f64 + minutes / 60.0;
        if self.north {
            dd
        } else {
            -dd
        }
    }

    /// Decimal degrees, positive east, negative west.
    pub fn longitude_deg(&self) -> f64 {
        let minutes = self.lon_thousandths_min as f64 / 1000.0;
        let dd = self.lon_deg as f64 + minutes / 60.0;
        if self.east {
            dd
        } else {
            -dd
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_mo_location() {
        // flags=0 (north, east), 37° 30.000' N, 122° 15.000' E, CEP=5 km
        // thousandths-of-minute: 30.000' -> 30000 (0x7530), 15.000' -> 15000 (0x3A98)
        let bytes: [u8; 11] = [
            0x00, 0x25, 0x75, 0x30, 0x7A, 0x3A, 0x98, 0x00, 0x00, 0x00, 0x05,
        ];

        let loc = MoLocation::parse(bytes).unwrap();
        assert!(loc.north && loc.east);
        assert_eq!(loc.lat_deg, 37);
        assert_eq!(loc.lat_thousandths_min, 30_000);
        assert_eq!(loc.lon_deg, 122);
        assert_eq!(loc.lon_thousandths_min, 15_000);
        assert_eq!(loc.cep_km, 5);

        let lat = loc.latitude_deg();
        let lon = loc.longitude_deg();
        assert!((lat - 37.5).abs() < 1e-9);
        assert!((lon - 122.25).abs() < 1e-9);
    }
}
