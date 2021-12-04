use std::ops::DerefMut;
use std::slice;
use uuid::Uuid;
use crate::bluetooth::uuid::BtUuid;
pub use zephyr_sys::raw::bt_data as ZBtData;
use crate::network::NetworkBufferSimple;

#[derive(Debug)]
pub enum BtData {
    Flags(u8),
    UuidAll(Vec<BtUuid>),
    CompleteName(String),
    CompleteNameStatic(&'static str),
}

impl BtData {
    fn type_number(&self) -> u8 {
        match self {
            BtData::Flags(_) => zephyr_sys::raw::BT_DATA_FLAGS as u8,
            BtData::UuidAll(_) => zephyr_sys::raw::BT_DATA_UUID128_ALL as u8,
            BtData::CompleteName(_) | BtData::CompleteNameStatic(_) => 0x09,
        }
    }

    fn data(&self) -> Vec<u8> {
        match self {
            BtData::Flags(flag) => flag.to_be_bytes().to_vec(),
            BtData::UuidAll(uuids) => uuids
                .iter()
                .flat_map(|uuid| uuid.as_bytes().into_iter().rev())
                .map(|byte| *byte)
                .collect(),
            BtData::CompleteName(name) => name.as_bytes().to_vec(),
            BtData::CompleteNameStatic(name) => name.as_bytes().to_vec(),
        }
    }

    pub fn raw(&self) -> RawBtData {
        RawBtData {
            type_: self.type_number(),
            data: self.data(),
        }
    }

    fn from_raw(data: &zephyr_sys::raw::bt_data) -> Option<BtData> {
        match data.type_ as u32 {
            zephyr_sys::raw::BT_DATA_UUID128_ALL => {
                let slice: &[u8] = unsafe { slice::from_raw_parts(data.data, data.data_len as usize) };
                let uuids = slice.chunks(16)
                    .map(|chunk| BtUuid::from_uuid(Uuid::from_slice_le(chunk).unwrap()))
                    .collect();
                Some(
                    BtData::UuidAll(uuids)
                )
            }
            zephyr_sys::raw::BT_DATA_FLAGS => {
                let data: u8 = unsafe { *data.data };
                Some(BtData::Flags(data))
            }
            _ => None,
        }
    }
}

pub struct RawBtData {
    type_: u8,
    data: Vec<u8>,
}

impl RawBtData {
    pub fn type_(&self) -> u8 {
        self.type_
    }
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

impl RawBtData {
    pub fn sys_ref(&self) -> ZBtData {
        ZBtData {
            type_: self.type_,
            data_len: self.data.len() as u8,
            data: self.data.as_ptr(),
        }
    }
}

pub extern "C" fn data_parser_callback(data: &mut zephyr_sys::raw::bt_data, parser: &mut DataParser<'_>) -> bool {
    let bt_data = BtData::from_raw(data);
    if let Some(bt_data) = bt_data {
        parser.data.push(bt_data);
    }
    true
}

pub struct DataParser<'data> {
    net_buf: &'data mut NetworkBufferSimple,
    data: Vec<BtData>,
}

impl DataParser<'_> {
    fn parse(&mut self) {
        unsafe {
            zephyr_sys::raw::bt_data_parse (
                std::mem::transmute(self.net_buf as *mut _),
                std::mem::transmute(data_parser_callback as *const fn (data: &mut zephyr_sys::raw::bt_data, parser: &mut DataParser<'_>) -> bool),
                std::mem::transmute(self as *mut _),
            )
        }
    }
}

pub trait ParseBtData: DerefMut<Target=NetworkBufferSimple> {
    fn parse_bt_data(&mut self) -> Vec<BtData> {
        let mut parser = DataParser {
            net_buf: self.deref_mut(),
            data: vec![],
        };
        parser.parse();
        parser.data
    }
}

impl<T> ParseBtData for T where T: DerefMut<Target=NetworkBufferSimple> {

}