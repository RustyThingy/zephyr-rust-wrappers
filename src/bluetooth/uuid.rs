use std::mem::transmute;
use crate::bluetooth::gatt::UserData;
use std::ops::Deref;
use uuid::{Bytes, Uuid};
pub use zephyr_sys::raw::{
    bt_uuid_128 as BtUuid128, bt_uuid_16 as BtUuid16, bt_uuid_32 as BtUuid32, bt_uuid,
};

pub static PRIMARY_SERVICE_UUID: BtUuid16 = uuid16(0x2800);
pub static GATT_CHARACTERISTIC_UUID: BtUuid16 =
    uuid16(zephyr_sys::raw::BT_UUID_GATT_CHRC_VAL as u16);
pub static GATT_CHARACTERISTIC_PRESENTATION_FORMAT_UUID: BtUuid16 =
    uuid16(zephyr_sys::raw::BT_UUID_GATT_CPF_VAL as u16);
pub static GATT_CLIENT_CHARACTERISTIC_CONFIGURATOR_UUID: BtUuid16 =
    uuid16(zephyr_sys::raw::BT_UUID_GATT_CCC_VAL as u16);

pub const BT_BASE_UUID: Uuid = Uuid::from_bytes([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5F, 0x9B, 0x34, 0xFB,
]);

const BT_BASE_D2: u16 = 0x0000;
const BT_BASE_D3: u16 = 0x1000;
const BT_BASE_D4: [u8; 8] = [0x80, 0x00, 0x00, 0x80, 0x5F, 0x9B, 0x34, 0xFB];

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq)]
pub struct BtUuid(Uuid);

unsafe impl UserData for BtUuid128 {}

impl BtUuid {
    pub const fn from_bytes(bytes: Bytes) -> BtUuid {
        BtUuid(Uuid::from_bytes(bytes))
    }
    pub const fn from_uuid(uuid: Uuid) -> BtUuid {
        let bytes: &[u8; 16] = uuid.as_bytes();
        // reverse byte order for bluetooth, unwrapped for const fn
        let mut rev_bytes = [0_u8; 16];
        rev_bytes[0] = bytes[15];
        rev_bytes[1] = bytes[14];
        rev_bytes[2] = bytes[13];
        rev_bytes[3] = bytes[12];
        rev_bytes[4] = bytes[11];
        rev_bytes[5] = bytes[10];
        rev_bytes[6] = bytes[9];
        rev_bytes[7] = bytes[8];
        rev_bytes[8] = bytes[7];
        rev_bytes[9] = bytes[6];
        rev_bytes[10] = bytes[5];
        rev_bytes[11] = bytes[4];
        rev_bytes[12] = bytes[3];
        rev_bytes[13] = bytes[2];
        rev_bytes[14] = bytes[1];
        rev_bytes[15] = bytes[0];
        Self::from_bytes(rev_bytes)
    }

    pub const fn service_uuid_32(service_id: u32) -> BtUuid {
        let d1_bytes = service_id.to_be_bytes();
        let d2_bytes = BT_BASE_D2.to_be_bytes();
        let d3_bytes = BT_BASE_D3.to_be_bytes();
        BtUuid(Uuid::from_bytes([
            d1_bytes[0],
            d1_bytes[1],
            d1_bytes[2],
            d1_bytes[3],
            d2_bytes[0],
            d2_bytes[1],
            d3_bytes[0],
            d3_bytes[1],
            BT_BASE_D4[0],
            BT_BASE_D4[1],
            BT_BASE_D4[2],
            BT_BASE_D4[3],
            BT_BASE_D4[4],
            BT_BASE_D4[5],
            BT_BASE_D4[6],
            BT_BASE_D4[7],
        ]))
    }

    pub const fn to_uuid128(&self) -> BtUuid128 {
        BtUuid128 {
            uuid: zephyr_sys::raw::bt_uuid {
                type_: zephyr_sys::raw::BT_UUID_TYPE_128 as u8,
            },
            val: *self.0.as_bytes(),
        }
    }
}

impl Deref for BtUuid {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BtUuid128> for BtUuid {
    fn from(bt_uuid_128: BtUuid128) -> Self {
        Self(Uuid::from_bytes(bt_uuid_128.val))
    }
}

impl From<BtUuid32> for BtUuid {
    fn from(bt_uuid_32: BtUuid32) -> Self {
        BtUuid::from(bt_uuid_32.val)
    }
}

impl From<BtUuid16> for BtUuid {
    fn from(bt_uuid_16: BtUuid16) -> Self {
        BtUuid::from(bt_uuid_16.val)
    }
}

impl From<u32> for BtUuid {
    fn from(number: u32) -> Self {
        Self(Uuid::from_fields(
            number,
            BT_BASE_D2,
            BT_BASE_D3,
            &BT_BASE_D4,
        ))
    }
}

impl From<u16> for BtUuid {
    fn from(number: u16) -> Self {
        Self(Uuid::from_fields(
            number as u32,
            BT_BASE_D2,
            BT_BASE_D3,
            &BT_BASE_D4,
        ))
    }
}

impl From<BtUuid> for BtUuid128 {
    fn from(bt_uuid: BtUuid) -> Self {
        let bytes = bt_uuid.0.as_bytes();
        BtUuid128 {
            uuid: zephyr_sys::raw::bt_uuid {
                type_: zephyr_sys::raw::BT_UUID_TYPE_128 as u8,
            },
            val: *bytes,
        }
    }
}

impl From<BtUuid> for BtUuid32 {
    fn from(bt_uuid: BtUuid) -> Self {
        let (d1, ..) = bt_uuid.0.to_fields_le();
        BtUuid32 {
            uuid: zephyr_sys::raw::bt_uuid {
                type_: zephyr_sys::raw::BT_UUID_TYPE_32 as u8,
            },
            val: d1,
        }
    }
}

impl From<BtUuid> for BtUuid16 {
    fn from(bt_uuid: BtUuid) -> Self {
        let (d1, ..) = bt_uuid.0.to_fields_le();
        BtUuid16 {
            uuid: zephyr_sys::raw::bt_uuid {
                type_: zephyr_sys::raw::BT_UUID_TYPE_16 as u8,
            },
            val: d1 as u16,
        }
    }
}

pub const fn uuid16(d1: u16) -> BtUuid16 {
    BtUuid16 {
        uuid: zephyr_sys::raw::bt_uuid {
            type_: zephyr_sys::raw::BT_UUID_TYPE_16 as u8,
        },
        val: d1,
    }
}

pub fn compare_uuids(one: &bt_uuid, other: &bt_uuid) -> bool {
    if one.type_ != other.type_ {
        false
    } else {
        match one.type_ as u32 {
            zephyr_sys::raw::BT_UUID_TYPE_128 => {
                let one: *const BtUuid128 = unsafe { transmute(one as *const bt_uuid) };
                let other: *const BtUuid128 = unsafe { transmute(other as *const bt_uuid) };

                unsafe { *one }.val == (unsafe { *other }.val)
            }
            zephyr_sys::raw::BT_UUID_TYPE_32 => {
                let one: *const BtUuid32 = unsafe { transmute(one as *const bt_uuid) };
                let other: *const BtUuid32 = unsafe { transmute(other as *const bt_uuid) };

                unsafe { *one }.val == unsafe { *other }.val
            }
            zephyr_sys::raw::BT_UUID_TYPE_16 => {
                let one: *const BtUuid16 = unsafe { transmute(one as *const bt_uuid) };
                let other: *const BtUuid16 = unsafe { transmute(other as *const bt_uuid) };

                unsafe { *one }.val == unsafe { *other }.val
            }
            type_ => unimplemented!("unknown uuid type {}", type_)
        }
    }
}