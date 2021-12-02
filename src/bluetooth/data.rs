use crate::bluetooth::uuid::BtUuid;
pub use zephyr_sys::raw::bt_data as ZBtData;

pub enum BtData<'val> {
    Flags(u8),
    UuidAll(&'val [BtUuid]),
    CompleteName(String),
    CompleteNameStatic(&'static str),
}

impl BtData<'_> {
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
