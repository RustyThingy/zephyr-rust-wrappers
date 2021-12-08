//! Syscalls and high level wrappers for the Zephyr Sensor API.
//!
//! Some functions in the high level API still are marked as `unsafe` as the required checks cannot be
//! performed to offer a safe API.
//!
//! ```should_panic
//!# use zephyr::context::Kernel as Context;
//! use zephyr_rust_wrappers::sensor::{Sensor, SensorChannel};
//!
//! let mut sensor = if let Some(sensor_device) = Context::device_get_binding("binding-label") {
//!     // we must make sure manually that we are binding to a sensor device
//!     unsafe {
//!         Sensor::new(sensor_device)
//!     }
//! } else {
//!     panic!("could not resolve binding for sensor")
//! };
//!
//! let value = sensor
//!                   .sample(SensorChannel::AmbientTemperature)
//!                   .expect("sampling value from sensor");
//!
//! println!("sensor measures {} °C", value as f32)
//! ```

pub use zephyr::device::Device;

use crate::{Context, ErrorNumber, ZephyrError};
pub use zephyr_sys::raw::sensor_trigger as SensorTrigger;
use zephyr_sys::raw::sensor_value as ZSensorValue;

const CONTEXT: SensorWrapperContext = SensorWrapperContext {};

/// Type that uses the internal Zephyr representation of a sensor value.
/// This type is needed to add additional functionality to a sensor value (e.g. [Into<f32>]).
///
/// As per the Zephyr documentation, the value can be converted to a floating point using the formula
/// <code>value = val1 + val2 * 10<sup>-6</sup></code>.
///
/// We define a [SensorValue] to be normalized if <code>0 <= val2 * 10<sup>-6</sup> < 1</code>. All
/// conversions that are defined for our implementation will always yield a normalized sensor value.
///
/// This type implements conversions to and from [i32] and [f32].
///
/// ```rust
///# use zephyr_rust_wrappers::sensor::SensorValue;
/// let sensor_float: SensorValue = 1.5.into();
/// let sensor_int: SensorValue = 1.into();
///
/// assert_eq!(1.5_f32, sensor_float.into());
/// assert_eq!(1_u32, sensor_float.into());
/// ```
#[derive(Copy, Clone, Debug)]
pub struct SensorValue {
    val1: i32,
    val2: i32,
}

impl From<ZSensorValue> for SensorValue {
    fn from(other: ZSensorValue) -> Self {
        Self {
            val1: other.val1,
            val2: other.val2,
        }
    }
}

impl From<SensorValue> for ZSensorValue {
    fn from(other: SensorValue) -> Self {
        ZSensorValue {
            val1: other.val1,
            val2: other.val2,
        }
    }
}

impl From<i32> for SensorValue {
    fn from(other: i32) -> Self {
        SensorValue {
            val1: other,
            val2: 0,
        }
    }
}

impl From<SensorValue> for i32 {
    fn from(other: SensorValue) -> Self {
        other.val1
    }
}

impl From<SensorValue> for f32 {
    fn from(other: SensorValue) -> Self {
        (other.val1 as f32) + (other.val2 as f32 * 1e-6_f32)
    }
}

impl From<f32> for SensorValue {
    fn from(other: f32) -> Self {
        let val1 = other.floor() as i32;
        let val2 = ((other - val1 as f32) * (1e6_f32)) as i32;
        Self { val1, val2 }
    }
}

/// Non-exhaustive list of sensor channels. The list uses the values from Zephyr header files and
/// might fail to compile if two or more sensor channels use the same representation.
#[repr(u32)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SensorChannel {
    AmbientTemperature = zephyr_sys::raw::sensor_channel_SENSOR_CHAN_AMBIENT_TEMP,
    Pressure = zephyr_sys::raw::sensor_channel_SENSOR_CHAN_PRESS,
}

/// Non-exhaustive list of sensor attributes. The list uses the values from Zephyr header files and
/// might fail to compile if two or more sensor channels use the same representation.
#[repr(u32)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SensorAttribute {
    LowerThreshold = zephyr_sys::raw::sensor_attribute_SENSOR_ATTR_LOWER_THRESH,
    UpperThreshold = zephyr_sys::raw::sensor_attribute_SENSOR_ATTR_UPPER_THRESH,
}

/// Non-exhaustive list of sensor trigger types. The list uses the values from Zephyr header files and
/// might fail to compile if two or more sensor channels use the same representation.
#[repr(u32)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TriggerType {
    Threshold = zephyr_sys::raw::sensor_trigger_type_SENSOR_TRIG_THRESHOLD,
    DataReady = zephyr_sys::raw::sensor_trigger_type_SENSOR_TRIG_DATA_READY,
}

/// Wrapper to the `sensor_sample_fetch_chan` syscall.
///
/// `device` MUST be a `'static` reference to a device descriptor obtained from the Zephyr API.
/// This wrapper calls the syscall and wraps the error number in a safe error type.
///
/// This method will fail if the sensor does not support `sensor_channel`.
pub unsafe fn sample_fetch_channel(
    device: &Device,
    sensor_channel: SensorChannel,
) -> Result<(), ZephyrError> {
    let errno = unsafe {
        zephyr_sys::syscalls::any::sensor_sample_fetch_chan(
            device as *const Device,
            sensor_channel as u32,
        )
    };

    if errno == 0 {
        Ok(())
    } else {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    }
}

/// Wrapper to the `sensor_channel_get` syscall.
///
/// `device` MUST be a `'static` reference to a device descriptor obtained from the Zephyr API.
/// This wrapper calls the syscall and wraps the error number in a safe error type.
///
/// This method will fail if the sensor does not support `sensor_channel`.
pub unsafe fn channel_get(
    device: &Device,
    sensor_channel: SensorChannel,
) -> Result<SensorValue, ZephyrError> {
    let mut z_sensor_value = ZSensorValue { val1: 0, val2: 0 };
    let errno = unsafe {
        zephyr_sys::syscalls::any::sensor_channel_get(
            device as *const Device,
            sensor_channel as u32,
            (&mut z_sensor_value) as *mut ZSensorValue,
        )
    };

    if errno == 0 {
        Ok(z_sensor_value.into())
    } else {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    }
}

/// Wrapper to the `sensor_channel_get` syscall.
///
/// `device` MUST be a `'static` reference to a device descriptor obtained from the Zephyr API.
/// This wrapper calls the syscall and wraps the error number in a safe error type.
///
/// This method might fail if the sensor driver does not expose this driver call or if the sensor
/// does not support the channel or the channel does not support the attribute.
pub unsafe fn attr_set(
    device: &Device,
    sensor_channel: SensorChannel,
    sensor_attribute: SensorAttribute,
    value: SensorValue,
) -> Result<(), ZephyrError> {
    let z_sensor_value: ZSensorValue = value.into();
    let errno = unsafe {
        zephyr_sys::syscalls::any::sensor_attr_set(
            device as *const Device,
            sensor_channel as u32,
            sensor_attribute as u32,
            (&z_sensor_value) as *const ZSensorValue,
        )
    };

    if errno == 0 {
        Ok(())
    } else {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    }
}

/// Implementation of the high level function `sensor_trigger_set` which is inlined in the Zephyr API.
/// The method essentially does the same as the original inline function but also checks if the driver
/// API is present in `device`.
///
/// `device` MUST be a `'static` reference to a device descriptor obtained from the Zephyr API.
/// This wrapper calls the syscall and wraps the error number in a safe error type.
///
/// This method might fail if the sensor driver does not expose this driver call or if the sensor
/// does not support the channel or the channel does not support the attribute.
pub unsafe fn trigger_set(
    device: &Device,
    sensor_trigger: &SensorTrigger,
    f: extern "C" fn(dev: &'static Device, trigger: &SensorTrigger),
) -> Result<(), ZephyrError> {
    use zephyr_sys::raw::sensor_driver_api as SensorDriverApi;
    // convert void pointer from C API to a sensor driver API Rust struct
    let api: Option<&SensorDriverApi> = std::mem::transmute(device.api);

    if let Some(api) = api {
        if let Some(trigger_set) = api.trigger_set {
            // convert safe Rust function pointer to pointer for binding. This can be done because
            // we use C calling convention for both functions (extern "C") and as per the Rustonomicon
            // a typed reference is effectively a (slim-)pointer. Using Option is not necessary because
            // the references are guaranteed to be non-null by Zephyr.
            let callback: extern "C" fn(dev: *const Device, trigger: *const SensorTrigger) =
                std::mem::transmute(f);
            // function pointers need to be called like this
            let errno = (trigger_set)(
                device as *const Device,
                sensor_trigger as *const SensorTrigger,
                Some(callback),
            );

            if errno == 0 {
                Ok(())
            } else {
                Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
            }
        } else {
            Err(ZephyrError::new_with_context(
                ErrorNumber::NotImplemented,
                &CONTEXT,
            ))
        }
    } else {
        Err(ZephyrError::new_with_context(
            ErrorNumber::NotImplemented,
            &CONTEXT,
        ))
    }
}

/// High level wrapper for a sensor.
///
/// This essentially wraps the static reference of the underlying device and offers methods that use
/// the safe syscall wrappers for an object oriented programming interface.
pub struct Sensor {
    device: &'static Device,
}

impl Sensor {
    /// Creates a new [Sensor] on the current interface.
    ///
    /// `device` MUST be a sensor representing a device. If `device` is not a sensor the behaviour
    /// when calling any method is undefined!
    pub unsafe fn new(device: &'static Device) -> Self {
        Sensor { device }
    }

    /// Fetch `channel` and then read the value from the internal buffer.
    ///
    /// This method might fail if the sensor does not support the requested channel.
    pub fn sample(&mut self, channel: SensorChannel) -> Result<SensorValue, ZephyrError> {
        // device is required to be a sensor device in constructor
        unsafe {
            sample_fetch_channel(self.device, channel)?;
            channel_get(self.device, channel)
        }
    }

    /// Set the attribute of the channel to the given sensor value.
    ///
    /// This method might fail if the sensor does not expose the driver API, the sensor channel or
    /// the channel does not support the sensor attribute.
    pub fn set_attr(
        &mut self,
        channel: SensorChannel,
        attribute: SensorAttribute,
        value: SensorValue,
    ) -> Result<(), ZephyrError> {
        // device is required to be a sensor device in constructor
        unsafe { attr_set(self.device, channel, attribute, value) }
    }

    /// Set the [SensorAttribute::LowerThreshold] attribute of `channel`. See [Sensor::set_attr] for details.
    pub fn set_lower_threshold(
        &mut self,
        channel: SensorChannel,
        value: SensorValue,
    ) -> Result<(), ZephyrError> {
        self.set_attr(channel, SensorAttribute::LowerThreshold, value)
    }

    /// Set the [SensorAttribute::UpperThreshold] attribute of `channel`. See [Sensor::set_attr] for details.
    pub fn set_upper_threshold(
        &mut self,
        channel: SensorChannel,
        value: SensorValue,
    ) -> Result<(), ZephyrError> {
        self.set_attr(channel, SensorAttribute::UpperThreshold, value)
    }

    /// Install a trigger of type `trigger_type` on `channel`.
    ///
    /// `f` is the callback function. The callback function will be called with the device the trigger
    /// has occurred on and the configuration that has been passed with the trigger.
    ///
    /// The callback uses a few tricks described in [The Nomicon](https://doc.rust-lang.org/nomicon/ffi.html#the-nullable-pointer-optimization)
    /// to allow the usage of Rust types in the callback. Also note that the callback [MUST NOT panic](https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-panics).
    pub fn enable_trigger(
        &mut self,
        trigger_type: TriggerType,
        channel: SensorChannel,
        f: extern "C" fn(dev: &'static Device, trigger: &SensorTrigger),
    ) -> Result<(), ZephyrError> {
        let sensor_trigger = SensorTrigger {
            type_: trigger_type as u32,
            chan: channel as u32,
        };
        /// device is required to be a sensor device in constructor
        unsafe {
            trigger_set(self.device, &sensor_trigger, f)
        }
    }
}

#[derive(Debug)]
struct SensorWrapperContext {}

impl Context for SensorWrapperContext {
    fn name(&self) -> &'static str {
        "sensor wrapper"
    }
}
