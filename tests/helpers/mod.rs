#![allow(dead_code)]
use usb_device::class::UsbClass;
use usbd_class_tester::prelude::*;

// State
pub const APP_IDLE: u8 = 0;
pub const APP_DETACH: u8 = 1;
pub const DFU_IDLE: u8 = 2;
pub const DFU_DNLOAD_SYNC: u8 = 3;
pub const DFU_DN_BUSY: u8 = 4;
pub const DFU_DNLOAD_IDLE: u8 = 5;
pub const DFU_MANIFEST_SYNC: u8 = 6;
pub const DFU_MANIFEST: u8 = 7;
pub const DFU_MANIFEST_WAIT_RESET: u8 = 8;
pub const DFU_UPLOAD_IDLE: u8 = 9;
pub const DFU_ERROR: u8 = 10;

pub const STATUS_OK: u8 = 0x00;
pub const STATUS_ERR_TARGET: u8 = 0x01;
pub const STATUS_ERR_FILE: u8 = 0x02;
pub const STATUS_ERR_WRITE: u8 = 0x03;
pub const STATUS_ERR_ERASE: u8 = 0x04;
pub const STATUS_ERR_CHECK_ERASED: u8 = 0x05;
pub const STATUS_ERR_PROG: u8 = 0x06;
pub const STATUS_ERR_VERIFY: u8 = 0x07;
pub const STATUS_ERR_ADDRESS: u8 = 0x08;
pub const STATUS_ERR_NOTDONE: u8 = 0x09;
pub const STATUS_ERR_FIRMWARE: u8 = 0x0A;
pub const STATUS_ERR_VENDOR: u8 = 0x0B;
pub const STATUS_ERR_USBR: u8 = 0x0C;
pub const STATUS_ERR_POR: u8 = 0x0D;
pub const STATUS_ERR_UNKNOWN: u8 = 0x0E;
pub const STATUS_ERR_STALLED_PKT: u8 = 0x0F;

pub trait HostExt<T> {
    fn upload(
        &mut self,
        cls: &mut T,
        block_num: u16,
        length: usize,
    ) -> core::result::Result<Vec<u8>, AnyUsbError>;
    fn ioop_raw(
        &mut self,
        cls: &mut T,
        reqt: CtrRequestType,
        req: u8,
        value: u16,
        index: u16,
        length: u16,
        data: Option<&[u8]>,
    ) -> core::result::Result<Vec<u8>, AnyUsbError>;
    fn ioop(
        &mut self,
        cls: &mut T,
        reqt: CtrRequestType,
        req: u8,
        value: u16,
        index: u16,
        length: u16,
        data: Option<&[u8]>,
    ) -> core::result::Result<Vec<u8>, AnyUsbError>;

    fn read(
        &mut self,
        cls: &mut T,
        req: u8,
        value: u16,
        index: u16,
        length: u16,
    ) -> core::result::Result<Vec<u8>, AnyUsbError>;
    fn write(
        &mut self,
        cls: &mut T,
        req: u8,
        value: u16,
        index: u16,
        length: u16,
        data: &[u8],
    ) -> core::result::Result<Vec<u8>, AnyUsbError>;

    fn download(
        &mut self,
        cls: &mut T,
        block_num: u16,
        data: &[u8],
    ) -> core::result::Result<Vec<u8>, AnyUsbError>;
    fn get_status(&mut self, cls: &mut T) -> core::result::Result<Vec<u8>, AnyUsbError>;
    fn clear_status(&mut self, cls: &mut T) -> core::result::Result<Vec<u8>, AnyUsbError>;
    fn get_state(&mut self, cls: &mut T) -> core::result::Result<Vec<u8>, AnyUsbError>;
    fn abort(&mut self, cls: &mut T) -> core::result::Result<Vec<u8>, AnyUsbError>;
}

impl<'a, T, M> HostExt<T> for Device<'a, T, M>
where
    T: UsbClass<EmulatedUsbBus>,
    M: UsbDeviceCtx<EmulatedUsbBus, T>,
{
    fn ioop_raw(
        &mut self,
        cls: &mut T,
        reqt: CtrRequestType,
        req: u8,
        value: u16,
        index: u16,
        length: u16,
        data: Option<&[u8]>,
    ) -> core::result::Result<Vec<u8>, AnyUsbError> {
        let mut buf: Vec<u8> = vec![0; length as usize];

        let setup = SetupPacket::new(reqt, req, value, index, length);

        let len = self.ep0(cls, setup, data, buf.as_mut_slice())?;
        buf.truncate(len);
        Ok(buf)
    }

    fn ioop(
        &mut self,
        cls: &mut T,
        reqt: CtrRequestType,
        req: u8,
        value: u16,
        index: u16,
        length: u16,
        data: Option<&[u8]>,
    ) -> core::result::Result<Vec<u8>, AnyUsbError> {
        self.ioop_raw(
            cls,
            reqt.class().interface(),
            req,
            value,
            index,
            length,
            data,
        )
    }

    fn read(
        &mut self,
        cls: &mut T,
        req: u8,
        value: u16,
        index: u16,
        length: u16,
    ) -> core::result::Result<Vec<u8>, AnyUsbError> {
        self.ioop(
            cls,
            CtrRequestType::to_host(),
            req,
            value,
            index,
            length,
            None,
        )
    }

    fn write(
        &mut self,
        cls: &mut T,
        req: u8,
        value: u16,
        index: u16,
        length: u16,
        data: &[u8],
    ) -> core::result::Result<Vec<u8>, AnyUsbError> {
        self.ioop(
            cls,
            CtrRequestType::to_device(),
            req,
            value,
            index,
            length,
            if data.len() > 0 { Some(data) } else { None },
        )
    }

    fn download(
        &mut self,
        cls: &mut T,
        block_num: u16,
        data: &[u8],
    ) -> core::result::Result<Vec<u8>, AnyUsbError> {
        if data.len() > u16::MAX as usize {
            return Err(AnyUsbError::DataConversion);
        }
        self.write(cls, 0x1, block_num, 0, data.len() as u16, data)
    }

    fn upload(
        &mut self,
        cls: &mut T,
        block_num: u16,
        length: usize,
    ) -> core::result::Result<Vec<u8>, AnyUsbError> {
        if length > u16::MAX as usize {
            return Err(AnyUsbError::DataConversion);
        }
        self.read(cls, 0x2, block_num, 0, length as u16)
    }

    fn get_status(&mut self, cls: &mut T) -> core::result::Result<Vec<u8>, AnyUsbError> {
        self.read(cls, 0x3, 0, 0, 6)
    }

    fn clear_status(&mut self, cls: &mut T) -> core::result::Result<Vec<u8>, AnyUsbError> {
        self.write(cls, 0x4, 0, 0, 0, &[])
    }

    fn get_state(&mut self, cls: &mut T) -> core::result::Result<Vec<u8>, AnyUsbError> {
        self.read(cls, 0x5, 0, 0, 1)
    }

    fn abort(&mut self, cls: &mut T) -> core::result::Result<Vec<u8>, AnyUsbError> {
        self.write(cls, 0x6, 0, 0, 0, &[])
    }
}

pub fn status(status: u8, poll_timeout: u32, state: u8) -> [u8; 6] {
    let t = poll_timeout.to_le_bytes();
    [status, t[0], t[1], t[2], state, 0]
}
