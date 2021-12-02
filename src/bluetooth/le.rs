use bitflags::bitflags;
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

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ConnectionParameters {
    pub interval_min: u16,
    pub interval_max: u16,
    pub latency: u16,
    pub timeout: u16,
}

impl From<zephyr_sys::raw::bt_le_conn_param> for ConnectionParameters {
    fn from(other: zephyr_sys::raw::bt_le_conn_param) -> Self {
        let zephyr_sys::raw::bt_le_conn_param {
            interval_min,
            interval_max,
            latency,
            timeout,
        } = other;
        Self {
            interval_min,
            interval_max,
            latency,
            timeout,
        }
    }
}

impl From<ConnectionParameters> for zephyr_sys::raw::bt_le_conn_param {
    fn from(other: ConnectionParameters) -> Self {
        let ConnectionParameters {
            interval_min,
            interval_max,
            latency,
            timeout,
        } = other;
        Self {
            interval_min,
            interval_max,
            latency,
            timeout,
        }
    }
}

#[repr(transparent)]
pub struct LeAddress(zephyr_sys::raw::bt_addr_le_t);

impl ToString for LeAddress {
    fn to_string(&self) -> String {
        let address: &[u8; 6] = &self.0.a.val;
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            address[0], address[1], address[2], address[3], address[4], address[5],
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
