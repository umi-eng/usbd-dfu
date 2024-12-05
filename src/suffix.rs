//! DFU file suffix

/// Firmware file suffix.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct Suffix {
    /// CRC checksum of the entire file, with the this CRC value set as zeroes.
    pub crc: u32,
    /// Length of this suffix.
    pub length: u8,
    /// DFU signature field. Must be `U`, `F`, `D`, i.e. "DFU" in ASCII, in reverse.
    pub dfu_signature: [u8; 3],
    /// BCD DFU specification number.
    pub dfu_specification: u16,
    /// USB vendor identifier.
    pub usb_vendor: u16,
    /// USB product identifier.
    pub usb_product: u16,
    /// BCD firmware release or version number.
    pub device: u16,
}

impl From<[u8; 16]> for Suffix {
    fn from(bytes: [u8; 16]) -> Self {
        Self {
            crc: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            length: bytes[4],
            dfu_signature: [bytes[5], bytes[6], bytes[7]],
            dfu_specification: u16::from_le_bytes([bytes[8], bytes[9]]),
            usb_vendor: u16::from_le_bytes([bytes[10], bytes[11]]),
            usb_product: u16::from_le_bytes([bytes[12], bytes[13]]),
            device: u16::from_le_bytes([bytes[14], bytes[15]]),
        }
    }
}
