use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::transmute;
use crate::bluetooth::connection::BtConnection;
use crate::bluetooth::uuid::BtUuid16;


#[repr(transparent)]
pub struct GattService<'attr>(zephyr_sys::raw::bt_gatt_service, PhantomData<&'attr ()>);

impl<'attr> GattService<'attr> {
    pub const fn new(attrs: &'attr mut [GattAttribute<'_, '_>]) -> Self {
        Self (
            zephyr_sys::raw::bt_gatt_service {
                attrs: unsafe { std::mem::transmute(attrs.as_mut_ptr()) },
                attr_count: attrs.len(),
                node: zephyr_sys::raw::sys_snode_t {
                    next: std::ptr::null_mut(),
                }
            },
            PhantomData,
        )
    }
}

pub type AttributeReadCallback = extern "C" fn(connection: &mut BtConnection, attribute: &GattAttribute, buf: *mut u8, len: u16, offset: u16) -> isize;
pub type AttributeWriteCallback = extern "C" fn(connection: &mut BtConnection, attribute: &GattAttribute, buf: *const u8, len: u16, offset: u16, flags: u8) -> isize;

pub extern "C" fn attribute_read_service(connection: &mut BtConnection, attribute: &GattAttribute, buf: *mut u8, len: u16, offset: u16) -> isize {
    println!("read attribute");
    unsafe {
        zephyr_sys::raw::bt_gatt_attr_read_service(
            transmute(connection),
            transmute(attribute),
            buf as *mut c_void,
            len,
            offset,
        )
    }
}

pub unsafe trait UserData {
}

unsafe impl UserData for Option<()> {

}

#[repr(transparent)]
pub struct GattAttribute<'uuid, 'ud>(zephyr_sys::raw::bt_gatt_attr, PhantomData<(&'uuid (), &'ud ())>);

impl<'uuid, 'ud> GattAttribute<'uuid, 'ud> {
    pub const fn new<U>(uuid: &'uuid BtUuid16, read: Option<AttributeReadCallback>, write: Option<AttributeWriteCallback>, user_data: &mut U, handle: u16, perm: u8) -> Self where U: UserData {
        Self(
            zephyr_sys::raw::bt_gatt_attr {
                uuid: unsafe { std::mem::transmute(uuid) },
                read: unsafe { std::mem::transmute(read) },
                write: unsafe { std::mem::transmute(write) },
                user_data: unsafe { std::mem::transmute(user_data as *mut _) },
                handle,
                perm,
            },
            PhantomData,
        )
    }
}