use crate::bluetooth::le::LeAddress;

#[repr(transparent)]
pub struct BtConnection(*mut zephyr_sys::raw::bt_conn);

impl BtConnection {
    pub fn get_destination(&self) -> &LeAddress {
        let address = unsafe { zephyr_sys::raw::bt_conn_get_dst(self.0) };
        unsafe { std::mem::transmute(address) }
    }
}
