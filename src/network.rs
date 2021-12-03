#[repr(transparent)]
pub struct NetworkBufferSimple (zephyr_sys::raw::net_buf_simple);

impl NetworkBufferSimple {
	pub fn new(data: &[u8], len: u16, size: u16) -> Self {
		Self (
			zephyr_sys::raw::net_buf_simple {
				data: unsafe {std::mem::transmute(data) },
				len: len,
				size: size,
				__buf: unsafe {std::mem::transmute(data)} // TODO is this intended?
			}
		)
	}
}
