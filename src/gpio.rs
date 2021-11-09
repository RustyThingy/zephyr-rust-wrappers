
pub use zephyr_sys::raw::device as Device;
pub use zephyr_sys::raw::{gpio_pin_t as GpioPin, gpio_flags_t as GpioFlags};

pub fn pin_configure(port: &Device, pin: GpioPin, flags: GpioFlags) -> Result<(), ()> {
    let ret = unsafe {
        zephyr_sys::syscalls::any::gpio_pin_configure(port as *const Device, pin, flags)
    };

    if ret == 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub fn pin_set_raw(port: &Device, pin: GpioPin, value: bool) -> Result<(), ()> {
    let ret = if value {
        unsafe {
            zephyr_sys::syscalls::any::gpio_port_set_bits_raw(port as *const Device, 1 << pin)
        }
    } else {
        unsafe {
            zephyr_sys::syscalls::any::gpio_port_clear_bits_raw(port as *const Device, 1 << pin)
        }
    };

    if ret == 0 {
        Ok(())
    } else {
        Err(())
    }
}