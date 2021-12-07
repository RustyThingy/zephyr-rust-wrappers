use crate::bluetooth::le::{AddressType, LeAddress};
use std::mem::transmute;

#[repr(transparent)]
#[derive(Eq)]
pub struct BtConnection(*mut zephyr_sys::raw::bt_conn);

impl BtConnection {
    pub fn get_destination(&self) -> Option<LeAddress> {
        let address = unsafe { zephyr_sys::raw::bt_conn_get_dst(transmute(self)).as_ref() };
        if let Some(address) = address {
            let address_clone = address.a.val.clone();
            Some(LeAddress::new(
                AddressType::from(address.type_),
                address_clone,
            ))
        } else {
            None
        }
    }
}

impl PartialEq for BtConnection {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}