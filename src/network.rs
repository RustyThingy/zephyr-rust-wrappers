use std::slice;
#[repr(transparent)]
pub struct NetworkBufferSimple(zephyr_sys::raw::net_buf_simple);

impl NetworkBufferSimple {
    pub fn new(data: &[u8], len: u16, size: u16) -> Self {
        Self(zephyr_sys::raw::net_buf_simple {
            data: unsafe { std::mem::transmute(data.as_ptr()) },
            len: len,
            size: size,
            __buf: unsafe { std::mem::transmute(data.as_ptr()) }, //TODO is this intended?
        })
    }

    pub fn data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.data, self.0.len as usize) }
    }
}
