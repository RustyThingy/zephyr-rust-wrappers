use crate::bluetooth::connection::BtConnection;
use crate::bluetooth::data::{BtData, RawBtData};
use crate::bluetooth::gatt::GattService;
use crate::bluetooth::le::{AddressWrapper, AdvertisementParameters, ConnectionParameters, ScanParameters};
use crate::bluetooth::CONTEXT;
use crate::{ErrorNumber, ZephyrError, ZephyrResult};
use crate::network::{NetworkBufferSimple};
use pretty_hex::simple_hex;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem::replace;
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::slice;

pub type BtReadyCallback = extern "C" fn(err: u32) -> ();

pub type BtConnectedCallback = extern "C" fn(connection: &mut BtConnection, error: u8);
pub type BtDisconnectedCallback = extern "C" fn(connection: &mut BtConnection, error: u8);
pub type BtLeParametersRequestedCallback =
    extern "C" fn(connection: &mut BtConnection, parameters: &mut ConnectionParameters) -> bool;
pub type BtLeParametersUpdatedCallback =
    extern "C" fn(connection: &mut BtConnection, interval: u16, latency: u16, timeout: u16);
pub type BtLeScanCallback = extern "C" fn(addr: &AddressWrapper, rssi: i8, adv_type: u8, buffer: &mut NetworkBufferSimple);

#[repr(transparent)]
pub struct BtConnectionCallbacks(zephyr_sys::raw::bt_conn_cb);

impl BtConnectionCallbacks {
    pub const fn new(
        connected: Option<BtConnectedCallback>,
        disconnected: Option<BtDisconnectedCallback>,
        parameters_requested: Option<BtLeParametersRequestedCallback>,
        parameters_updated: Option<BtLeParametersUpdatedCallback>,
    ) -> Self {
        let connected = unsafe { std::mem::transmute(connected) };
        let disconnected = unsafe { std::mem::transmute(disconnected) };
        let le_param_req = unsafe { std::mem::transmute(parameters_requested) };
        let le_param_updated = unsafe { std::mem::transmute(parameters_updated) };

        Self(zephyr_sys::raw::bt_conn_cb {
            connected,
            disconnected,
            le_param_req,
            le_param_updated,
            _next: std::ptr::null_mut(),
        })
    }

    fn inner_ptr_mut(&mut self) -> *mut zephyr_sys::raw::bt_conn_cb {
        (&mut self.0) as *mut _
    }
}

struct ApiContainer {
    api: Option<Api>,
}

impl ApiContainer {
    fn take_api(&mut self) -> Api {
        let api = replace(&mut self.api, None);
        api.unwrap()
    }
}

static mut API_CONTAINER: ApiContainer = ApiContainer { api: Some(Api {}) };

/// Only one instance is allowed to exist!
pub struct Api;

impl Api {
    pub fn register_service(service: &mut GattService) -> ZephyrResult<()> {
        let errno =
            unsafe { zephyr_sys::raw::bt_gatt_service_register(std::mem::transmute(service)) };

        if errno == 0 {
            Ok(())
        } else {
            Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
        }
    }

    pub fn enable() -> Result<Api, ZephyrError> {
        let api = unsafe { API_CONTAINER.take_api() };
        unsafe {
            enable(None)?;
        }

        Ok(api)
    }

    pub fn set_name(&mut self, name: &str) -> Result<(), ZephyrError> {
        unsafe { set_name(name) }
    }

    pub fn register_connection_callbacks<'api, 'cb: 'api>(
        &'api mut self,
        callbacks: &'cb mut BtConnectionCallbacks,
    ) {
        unsafe { register_connection_callbacks(callbacks) }
    }

    pub fn start_advertising(
        &mut self,
        parameters: &AdvertisementParameters,
        advertisement_data: Option<&[BtData]>,
        scan_response_data: Option<&[BtData]>,
    ) -> ZephyrResult<()> {
        unsafe { start_advertising(parameters, advertisement_data, scan_response_data) }
    }

    pub fn start_scanning(
        &mut self,
        parameters: &ScanParameters,
        callback: BtLeScanCallback,
    ) -> ZephyrResult<()> {
        unsafe { start_scanning(parameters, callback) }
    }
}

pub unsafe fn enable(callback: Option<BtReadyCallback>) -> Result<(), ZephyrError> {
    let callback: zephyr_sys::raw::bt_ready_cb_t = std::mem::transmute(callback);
    let errno = zephyr_sys::raw::bt_enable(callback);

    if errno != 0 {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    } else {
        Ok(())
    }
}

pub unsafe fn set_name(name: &str) -> Result<(), ZephyrError> {
    let c_str = CString::new(name)
        .map_err(|e| ZephyrError::new_with_context(ErrorNumber::NotImplemented, &CONTEXT))?;

    let errno = zephyr_sys::raw::bt_set_name(c_str.as_ptr());
    if errno != 0 {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    } else {
        Ok(())
    }
}

pub unsafe fn register_connection_callbacks(callbacks: &mut BtConnectionCallbacks) {
    zephyr_sys::raw::bt_conn_cb_register(callbacks.inner_ptr_mut());
}

pub unsafe fn start_advertising(
    parameters: &AdvertisementParameters,
    advertisement_data: Option<&[BtData]>,
    scan_response_data: Option<&[BtData]>,
) -> ZephyrResult<()> {
    let advertisement_handle =
        RawAdvertisementHandle::new(parameters, advertisement_data, scan_response_data);

    let errno = zephyr_sys::raw::bt_le_adv_start(
        advertisement_handle.param_ptr(),
        advertisement_handle.ad_ptr(),
        advertisement_handle.ad_len(),
        advertisement_handle.sd_ptr(),
        advertisement_handle.sd_len(),
    );

    if errno == 0 {
        Ok(())
    } else {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    }
}

struct RawAdvertisementHandle {
    raw_ad_data: Option<Vec<RawBtData>>,
    raw_sd_data: Option<Vec<RawBtData>>,
    zraw_ad_data: Option<Vec<zephyr_sys::raw::bt_data>>,
    zraw_sd_data: Option<Vec<zephyr_sys::raw::bt_data>>,
    adv_param: zephyr_sys::raw::bt_le_adv_param,
}

impl RawAdvertisementHandle {
    fn new(
        parameters: &AdvertisementParameters,
        advertisement_data: Option<&[BtData]>,
        scan_response_data: Option<&[BtData]>,
    ) -> Self {
        let adv_param = parameters.into();
        let raw_ad_data = advertisement_data.map(|slice| {
            slice
                .iter()
                .map(|bt_data| {
                    let raw = bt_data.raw();
                    println!(
                        "type: 0x{:02x} len: {:02} data: {}",
                        raw.type_(),
                        raw.data().len(),
                        simple_hex(raw.data())
                    );
                    raw
                })
                .collect()
        });
        let raw_sd_data =
            scan_response_data.map(|slice| slice.iter().map(|bt_data| bt_data.raw()).collect());

        let zraw_ad_data = raw_ad_data.as_ref().map(|vec: &Vec<RawBtData>| {
            vec.iter()
                .map(|raw_bt_data| raw_bt_data.sys_ref())
                .collect()
        });
        let zraw_sd_data = raw_sd_data.as_ref().map(|vec: &Vec<RawBtData>| {
            vec.iter()
                .map(|raw_bt_data| raw_bt_data.sys_ref())
                .collect()
        });

        RawAdvertisementHandle {
            raw_ad_data,
            raw_sd_data,
            zraw_ad_data,
            zraw_sd_data,
            adv_param,
        }
    }

    fn param_ptr(&self) -> *const zephyr_sys::raw::bt_le_adv_param {
        (&self.adv_param) as *const _
    }

    fn ad_ptr(&self) -> *const zephyr_sys::raw::bt_data {
        match self.zraw_ad_data.as_ref() {
            Some(ad) => ad.as_ptr(),
            _ => std::ptr::null(),
        }
    }

    fn ad_len(&self) -> usize {
        self.zraw_ad_data
            .as_ref()
            .map(|vec| vec.len())
            .unwrap_or(0_usize)
    }

    fn sd_ptr(&self) -> *const zephyr_sys::raw::bt_data {
        match self.zraw_sd_data.as_ref() {
            Some(sd) => sd.as_ptr(),
            _ => std::ptr::null(),
        }
    }

    fn sd_len(&self) -> usize {
        self.zraw_sd_data
            .as_ref()
            .map(|vec| vec.len())
            .unwrap_or(0_usize)
    }
}

pub unsafe fn start_scanning(scan_parameters: &ScanParameters, callback: BtLeScanCallback) -> ZephyrResult<()> {
    let bt_le_scan_param = zephyr_sys::raw::bt_le_scan_param::from(scan_parameters);

    let errno = zephyr_sys::raw::bt_le_scan_start(
        &bt_le_scan_param,
        std::mem::transmute(callback),
    );

    if errno == 0 {
        Ok(())
    } else {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    }
}