use std::ops::Deref;
use uuid::{Bytes, Uuid};
pub use zephyr_sys::raw::{
    bt_uuid_128 as BtUuid128, bt_uuid_16 as BtUuid16, bt_uuid_32 as BtUuid32,
};

pub const BT_BASE_UUID: Uuid = Uuid::from_bytes([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5F, 0x9B, 0x34, 0xFB,
]);

const BT_BASE_D2: u16 = 0x0000;
const BT_BASE_D3: u16 = 0x1000;
const BT_BASE_D4: [u8; 8] = [0x80, 0x00, 0x00, 0x80, 0x5F, 0x9B, 0x34, 0xFB];

#[repr(transparent)]
pub struct BtUuid(Uuid);

impl BtUuid {
    pub const fn from_bytes(bytes: Bytes) -> BtUuid {
        BtUuid(Uuid::from_bytes(bytes))
    }
    pub const fn from_uuid(uuid: Uuid) -> BtUuid {
        BtUuid(uuid)
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
