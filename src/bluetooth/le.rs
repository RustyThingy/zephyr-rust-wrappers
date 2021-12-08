use bitflags::bitflags;
use std::fmt::{write, Debug, Display, Formatter};
bitflags! {
    pub struct AdvertisementFlags: u8 {
        const GeneralDiscoverable = zephyr_sys::raw::BT_LE_AD_GENERAL as u8;
        const LimitedDiscoverable = zephyr_sys::raw::BT_LE_AD_LIMITED as u8;
        const NoBREDRSupport = zephyr_sys::raw::BT_LE_AD_NO_BREDR as u8;
    }
}

bitflags! {
    pub struct AdvertisementOptions : u32 {
        const None = zephyr_sys::raw::BT_LE_ADV_OPT_NONE;
        const Connectable = zephyr_sys::raw::BT_LE_ADV_OPT_CONNECTABLE;
        const UseName = zephyr_sys::raw::BT_LE_ADV_OPT_USE_NAME;
        const ForceNameInAdvertisement = zephyr_sys::raw::BT_LE_ADV_OPT_FORCE_NAME_IN_AD;
    }
}

bitflags! {
    pub struct ScanOptions: u32 {
        const None = zephyr_sys::raw::BT_LE_SCAN_OPT_NONE;
        const FilterDuplicate = zephyr_sys::raw::BT_LE_SCAN_OPT_FILTER_DUPLICATE;
        const FilterAcceptList = zephyr_sys::raw::BT_LE_SCAN_OPT_FILTER_ACCEPT_LIST;
        const Coded = zephyr_sys::raw::BT_LE_SCAN_OPT_CODED;
        const No1M = zephyr_sys::raw::BT_LE_SCAN_OPT_NO_1M;
    }
}

bitflags! {
    pub struct ScanType: u8 {
        const Passive = zephyr_sys::raw::BT_LE_SCAN_TYPE_PASSIVE as u8;
        const Active = zephyr_sys::raw::BT_LE_SCAN_TYPE_ACTIVE as u8;
    }
}

#[repr(transparent)]
pub struct ConnectionParameters(zephyr_sys::raw::bt_le_conn_param);

impl ConnectionParameters {
    pub const fn default() -> ConnectionParameters {
        ConnectionParameters {
            0: zephyr_sys::raw::bt_le_conn_param {
                interval_min: 0x0018,
                interval_max: 0x0028,
                latency: 0,
                timeout: 400,
            },
        }
    }
}

#[repr(transparent)]
pub struct ConnectionCreationParameters(zephyr_sys::raw::bt_conn_le_create_param);

impl ConnectionCreationParameters {
    pub const fn default() -> ConnectionCreationParameters {
        ConnectionCreationParameters {
            0: zephyr_sys::raw::bt_conn_le_create_param {
                options: 0,
                interval: 0x0060,
                window: 0x0030,
                interval_coded: 0,
                window_coded: 0,
                timeout: 0,
            },
        }
    }
}

#[repr(u8)]
pub enum AddressType {
    Public = zephyr_sys::raw::BT_ADDR_LE_PUBLIC as u8,
    Random = zephyr_sys::raw::BT_ADDR_LE_RANDOM as u8,
    PublicId = zephyr_sys::raw::BT_ADDR_LE_PUBLIC_ID as u8,
    RandomId = zephyr_sys::raw::BT_ADDR_LE_RANDOM_ID as u8,
    Other(u8),
}

impl From<u8> for AddressType {
    fn from(number: u8) -> Self {
        match number as u32 {
            zephyr_sys::raw::BT_ADDR_LE_PUBLIC => AddressType::Public,
            zephyr_sys::raw::BT_ADDR_LE_RANDOM => AddressType::Random,
            zephyr_sys::raw::BT_ADDR_LE_PUBLIC_ID => AddressType::PublicId,
            zephyr_sys::raw::BT_ADDR_LE_RANDOM_ID => AddressType::RandomId,
            other => AddressType::Other(other as u8),
        }
    }
}

impl Display for AddressType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressType::Public => write!(f, "public"),
            AddressType::Random => write!(f, "random"),
            AddressType::PublicId => write!(f, "public-id"),
            AddressType::RandomId => write!(f, "random-id"),
            AddressType::Other(other) => write!(f, "unknown: 0x{:02x}", other),
        }
    }
}

#[repr(transparent)]
pub struct AddressWrapper(zephyr_sys::raw::bt_addr_le_t);

impl AddressWrapper {
    pub fn address(&self) -> &[u8] {
        &self.0.a.val
    }
}

impl Debug for AddressWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let address: &[u8; 6] = &self.0.a.val;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} ({})",
            address[5],
            address[4],
            address[3],
            address[2],
            address[1],
            address[0],
            AddressType::from(self.0.type_),
        )
    }
}

pub struct LeAddress {
    address: [u8; 6],
    addr_type: AddressType,
}

impl LeAddress {
    pub fn new(addr_type: AddressType, address: [u8; 6]) -> Self {
        Self { addr_type, address }
    }
}

impl Debug for LeAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let address: &[u8; 6] = &self.address;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} ({})",
            address[5], address[4], address[3], address[2], address[1], address[0], self.addr_type,
        )
    }
}

impl Display for LeAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let address: &[u8; 6] = &self.address;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} ({})",
            address[5], address[4], address[3], address[2], address[1], address[0], self.addr_type,
        )
    }
}

pub struct AdvertisementParameters {
    id: u8,
    sid: u8,
    secondary_max_skip: u8,
    options: AdvertisementOptions,
    interval_min: u32,
    interval_max: u32,
    peer: Option<LeAddress>,
}

impl AdvertisementParameters {
    pub fn new(
        id: u8,
        sid: u8,
        secondary_max_skip: u8,
        options: AdvertisementOptions,
        interval_min: u32,
        interval_max: u32,
        peer: Option<LeAddress>,
    ) -> Self {
        AdvertisementParameters {
            id,
            sid,
            secondary_max_skip,
            options,
            interval_min,
            interval_max,
            peer,
        }
    }
}

impl From<&AdvertisementParameters> for zephyr_sys::raw::bt_le_adv_param {
    fn from(other: &AdvertisementParameters) -> Self {
        let AdvertisementParameters {
            id,
            sid,
            secondary_max_skip,
            options,
            interval_min,
            interval_max,
            peer,
        } = other;
        Self {
            id: *id,
            sid: *sid,
            secondary_max_skip: *secondary_max_skip,
            options: options.bits(),
            interval_min: *interval_min,
            interval_max: *interval_max,
            peer: unsafe { std::mem::transmute(peer.as_ref()) },
        }
    }
}

pub struct ScanParameters {
    type_: ScanType,
    options: ScanOptions,
    interval: u16,
    window: u16,
    timeout: u16,
    interval_coded: u16,
    window_coded: u16,
}

impl ScanParameters {
    pub const fn new(
        type_: ScanType,
        options: ScanOptions,
        interval: u16,
        window: u16,
        timeout: u16,
        interval_coded: u16,
        window_coded: u16,
    ) -> Self {
        Self {
            type_,
            options,
            interval,
            window,
            timeout,
            interval_coded,
            window_coded,
        }
    }
}

impl From<&ScanParameters> for zephyr_sys::raw::bt_le_scan_param {
    fn from(other: &ScanParameters) -> Self {
        let ScanParameters {
            type_,
            options,
            interval,
            window,
            timeout,
            interval_coded,
            window_coded,
        } = other;
        Self {
            type_: type_.bits(),
            interval: *interval,
            window: *window,
            options: options.bits(),
            timeout: *timeout,
            window_coded: *window_coded,
            interval_coded: *interval_coded,
        }
    }
}
