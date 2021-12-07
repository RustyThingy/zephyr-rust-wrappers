use crate::bluetooth::connection::BtConnection;
use crate::bluetooth::uuid::{BtUuid, BtUuid128, BtUuid16};
use crate::bluetooth::CONTEXT;
use crate::{ZephyrError, ZephyrResult};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::transmute;

#[macro_export]
macro_rules! gatt_attribute {
    ($uuid: expr, $permission: expr, $read_cb: expr, $write_cb: expr, $value: expr) => {
        GattAttribute::with_raw($uuid, $read_cb, $write_cb, $value, 0, $permission)
    };
}

#[macro_export]
macro_rules! gatt_characteristic {
    ($uuid: expr, $properties: expr, $permission: expr, $read_cb: expr, $write_cb: expr, $value: expr) =>
    {
        gatt_attribute!(
            crate::bluetooth::uuid::uuid16(zephyr_sys::raw::BT_UUID_GATT_CHRC),
            zephyr_sys::raw::BT_GATT_PERM_HEAD,
            Some(attribute_read_characteristic),
            None,
            Box::into_raw(Box::new(zephyr_sys::raw::bt_gatt_chrc {
                uuid: $uuid,
                value_handle: 0,
                properties: $properties,
            })),
        ),
        gatt_attribute!(
            $uuid,
            $permission,
            $read_cb,
            $write_cb,
            $value
        )
    }
}

#[repr(transparent)]
pub struct GattService<'attr>(zephyr_sys::raw::bt_gatt_service, PhantomData<&'attr ()>);

impl<'attr> GattService<'attr> {
    pub const fn new(attrs: &'attr mut [GattAttribute<'_, '_>]) -> Self {
        Self(
            zephyr_sys::raw::bt_gatt_service {
                attrs: unsafe { std::mem::transmute(attrs.as_ptr()) },
                attr_count: attrs.len(),
                node: zephyr_sys::raw::sys_snode_t {
                    next: std::ptr::null_mut(),
                },
            },
            PhantomData,
        )
    }
}

#[derive(Copy, Clone)]
pub union AttributeReadCallback {
    pub rust: extern "C" fn(
        connection: &mut BtConnection,
        attribute: &GattAttribute,
        buf: *mut u8,
        len: u16,
        offset: u16,
    ) -> isize,
    pub c: unsafe extern "C" fn(
        conn: *mut zephyr_sys::raw::bt_conn,
        attr: *const zephyr_sys::raw::bt_gatt_attr,
        buf: *mut std::ffi::c_void,
        len: u16,
        offset: u16,
    ) -> isize,
}

#[derive(Copy, Clone)]
pub union AttributeWriteCallback {
    pub rust: extern "C" fn(
        connection: &mut BtConnection,
        attribute: &GattAttribute,
        buf: *const u8,
        len: u16,
        offset: u16,
        flags: u8,
    ) -> isize,
    pub c: unsafe extern "C" fn(
        conn: *mut zephyr_sys::raw::bt_conn,
        attr: *const zephyr_sys::raw::bt_gatt_attr,
        buf: *const std::ffi::c_void,
        len: u16,
        offset: u16,
        flags: u8,
    ) -> isize,
}

macro_rules! attribute_read {
    ($rust_api: ident, $c_api: ident) => {
        pub extern "C" fn $rust_api(
            connection: &mut BtConnection,
            attribute: &GattAttribute,
            buf: *mut u8,
            len: u16,
            offset: u16,
        ) -> isize {
            unsafe {
                zephyr_sys::raw::$c_api(
                    transmute(connection),
                    transmute(attribute),
                    buf as *mut c_void,
                    len,
                    offset,
                )
            }
        }
    };
}

attribute_read!(attribute_read_service, bt_gatt_attr_read_service);
attribute_read!(attribute_read_characteristic, bt_gatt_attr_read_chrc);

pub unsafe trait UserData {}

unsafe impl UserData for Option<()> {}

pub const FIRST_ATTRIBUTE_HANDLE : u16 = zephyr_sys::raw::BT_ATT_FIRST_ATTRIBUTE_HANDLE as u16;
pub const LAST_ATTRIBUTE_HANDLE : u16 = zephyr_sys::raw::BT_ATT_LAST_ATTRIBUTE_HANDLE as u16;
pub const GATT_DISCOVER_PRIMARY : u8 = zephyr_sys::raw::BT_GATT_DISCOVER_PRIMARY as u8;
pub const GATT_DISCOVER_SECONDARY : u8 = zephyr_sys::raw::BT_GATT_DISCOVER_SECONDARY as u8;
pub const GATT_DISCOVER_INCLUDE : u8 = zephyr_sys::raw::BT_GATT_DISCOVER_INCLUDE as u8;
pub const GATT_DISCOVER_DESCRIPTOR : u8 = zephyr_sys::raw::BT_GATT_DISCOVER_DESCRIPTOR as u8;
pub const GATT_DISCOVER_STD_CHAR_DESC : u8 = zephyr_sys::raw::BT_GATT_DISCOVER_STD_CHAR_DESC as u8;
pub const GATT_ITER_STOP : u8 = zephyr_sys::raw::BT_GATT_ITER_STOP as u8;
pub const GATT_ITER_CONTINUE : u8 = zephyr_sys::raw::BT_GATT_ITER_CONTINUE as u8;

#[repr(transparent)]
pub struct GattAttribute<'uuid, 'ud>(
    zephyr_sys::raw::bt_gatt_attr,
    PhantomData<(&'uuid (), &'ud ())>,
);

impl<'uuid, 'ud> GattAttribute<'uuid, 'ud> {
    pub const fn new<U>(
        uuid: &'uuid BtUuid16,
        read: Option<AttributeReadCallback>,
        write: Option<AttributeWriteCallback>,
        user_data: &mut U,
        handle: u16,
        perm: u8,
    ) -> Self
    where
        U: UserData,
    {
        Self(
            zephyr_sys::raw::bt_gatt_attr {
                uuid: unsafe { std::mem::transmute(uuid) },
                read: match read {
                    None => None,
                    Some(read) => Some(unsafe { std::mem::transmute(read) }),
                },
                write: match write {
                    None => None,
                    Some(write) => Some(unsafe { std::mem::transmute(write) }),
                },
                user_data: unsafe { std::mem::transmute(user_data as *mut _) },
                handle,
                perm,
            },
            PhantomData,
        )
    }

    pub const fn with_raw(
        uuid: *const zephyr_sys::raw::bt_uuid,
        read: Option<AttributeReadCallback>,
        write: Option<AttributeWriteCallback>,
        user_data: *mut c_void,
        handle: u16,
        perm: u8,
    ) -> Self {
        Self(
            zephyr_sys::raw::bt_gatt_attr {
                uuid,
                read: match read {
                    None => None,
                    Some(read) => Some(unsafe { transmute(read) }),
                },
                write: match write {
                    None => None,
                    Some(write) => Some(unsafe { std::mem::transmute(write) }),
                },
                user_data,
                handle,
                perm,
            },
            PhantomData,
        )
    }
}

#[repr(transparent)]
pub struct NotifyParams(zephyr_sys::raw::bt_gatt_notify_params);

impl NotifyParams {
    pub fn by_uuid(attribute: &BtUuid128, data: &[u8]) -> Self {
        Self(zephyr_sys::raw::bt_gatt_notify_params {
            uuid: unsafe { transmute(attribute as *const _) },
            attr: std::ptr::null(),
            data: data.as_ptr() as *const std::ffi::c_void,
            len: data.len() as u16,
            func: None,
            user_data: std::ptr::null_mut(),
        })
    }
}

pub unsafe fn notify(
    connection: Option<&mut BtConnection>,
    params: &mut NotifyParams,
) -> ZephyrResult<()> {
    let result = unsafe {
        zephyr_sys::raw::bt_gatt_notify_cb(transmute(connection), transmute(params as *mut _))
    };

    if result == 0 {
        Ok(())
    } else {
        Err(ZephyrError::from_errno_with_context(result, &CONTEXT))
    }
}

pub type DiscoverCallback = extern "C" fn(
    conn: &mut BtConnection,
    attr: Option<&GattAttribute>,
    params: &mut DiscoverParameters,
) -> u8;

#[repr(transparent)]
pub struct DiscoverParameters(zephyr_sys::raw::bt_gatt_discover_params);

impl DiscoverParameters {
    pub const fn new(
        uuid: &BtUuid128,
        discover_cb: DiscoverCallback,
        start_handle: u16,
        end_handle: u16,
        type_: u8,
    ) -> DiscoverParameters {
        DiscoverParameters (
            zephyr_sys::raw::bt_gatt_discover_params {
                uuid: unsafe { transmute(uuid) },
                func: unsafe { transmute(discover_cb) },
                __bindgen_anon_1: zephyr_sys::raw::bt_gatt_discover_params__bindgen_ty_1 {
                    start_handle,
                },
                end_handle,
                type_,
            }
        )
    }
}

pub unsafe fn discover(connection: &mut BtConnection, parameters: &mut DiscoverParameters) -> ZephyrResult<()> {
    let errno = zephyr_sys::raw::bt_gatt_discover(
        transmute(connection),
        transmute(parameters),
    );

    if errno == 0 {
        Ok(())
    } else {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    }
}