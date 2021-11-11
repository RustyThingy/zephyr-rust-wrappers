//! Syscalls and high level wrappers for the Zephyr GPIO API.
//!
//! Some functions in the high level API still are marked as `unsafe` as the required checks cannot be
//! performed to offer a safe API.

use crate::{Context, ZephyrError};
pub use zephyr_sys::raw::device as Device;
pub use zephyr_sys::raw::{gpio_flags_t as GpioFlags, gpio_pin_t as GpioPinNumber};

const CONTEXT: GpioWrapperContext = GpioWrapperContext {};

/// Safe wrapper for the `gpio_pin_configure` syscall.
///
/// `device` MUST be a `'static` reference to a device descriptor obtained from the Zephyr API.
/// This wrapper calls the syscall and wraps the error number in a safe error type.
///
/// This method might fail/panic/abort if the device is not a gpio device.
pub unsafe fn pin_configure(
    port: &Device,
    pin: GpioPinNumber,
    flags: GpioFlags,
) -> Result<(), ZephyrError> {
    let errno = zephyr_sys::syscalls::any::gpio_pin_configure(port as *const Device, pin, flags);

    if errno == 0 {
        Ok(())
    } else {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    }
}

/// Safe wrapper for the `gpio_pin_set_raw` syscall.
///
/// `device` MUST be a `'static` reference to a device descriptor obtained from the Zephyr API.
/// This wrapper calls the syscall and wraps the error number in a safe error type.
///
/// This method might fail/panic/abort if the device is not a gpio device.
pub unsafe fn pin_set_raw(
    port: &Device,
    pin: GpioPinNumber,
    value: bool,
) -> Result<(), ZephyrError> {
    let errno = if value {
        zephyr_sys::syscalls::any::gpio_port_set_bits_raw(port as *const Device, 1 << pin)
    } else {
        zephyr_sys::syscalls::any::gpio_port_clear_bits_raw(port as *const Device, 1 << pin)
    };

    if errno == 0 {
        Ok(())
    } else {
        Err(ZephyrError::from_errno_with_context(errno, &CONTEXT))
    }
}

#[derive(Debug)]
struct GpioWrapperContext {}

impl Context for GpioWrapperContext {
    fn name(&self) -> &'static str {
        "gpio wrapper context"
    }
}

/// High level wrapper for a GPIO pin.
pub struct GpioPin {
    device: &'static Device,
    pin_number: GpioPinNumber,
}

impl GpioPin {
    /// Creates a new [GpioPin] on the current interface.
    ///
    /// `device` MUST be a gpio device. If `device` is not a gpio device the behaviour
    /// when calling any method is undefined!
    pub unsafe fn new(
        device: &'static Device,
        pin_number: GpioPinNumber,
        flags: GpioFlags,
    ) -> Result<Self, ZephyrError> {
        pin_configure(device, pin_number, flags)?;
        Ok(GpioPin { device, pin_number })
    }

    /// Set the state of the GPIO pin.
    pub fn set_value(&mut self, value: bool) -> Result<(), ZephyrError> {
        // device MUST BE a gpio device as per the constructor
        unsafe { pin_set_raw(self.device, self.pin_number, value) }
    }
}
