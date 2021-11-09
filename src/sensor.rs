
pub use zephyr::device::Device;

use zephyr_sys::raw::sensor_value as ZSensorValue;
pub use zephyr_sys::raw::sensor_trigger as SensorTrigger;

#[derive(Copy, Clone)]
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

impl From<SensorValue> for f32 {
    fn from(other: SensorValue) -> Self {
        (other.val1 as f32) + (other.val2 as f32 * 1e-6_f32)
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum SensorChannel {
    AmbientTemperature = zephyr_sys::raw::sensor_channel_SENSOR_CHAN_AMBIENT_TEMP,
    Pressure = zephyr_sys::raw::sensor_channel_SENSOR_CHAN_PRESS,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum SensorAttribute {
    LowerThreshold = zephyr_sys::raw::sensor_attribute_SENSOR_ATTR_LOWER_THRESH,
    UpperThreshold = zephyr_sys::raw::sensor_attribute_SENSOR_ATTR_UPPER_THRESH,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum TriggerType {
    Threshold = zephyr_sys::raw::sensor_trigger_type_SENSOR_TRIG_THRESHOLD,
    DataReady = zephyr_sys::raw::sensor_trigger_type_SENSOR_TRIG_DATA_READY,
}

pub fn sample_fetch_channel(device: &Device, sensor_channel: SensorChannel) -> Result<(), ()> {
    let err = unsafe {
        zephyr_sys::syscalls::any::sensor_sample_fetch_chan(device as *const Device, sensor_channel as u32)
    };

    if err == 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub fn channel_get(device: &Device, sensor_channel: SensorChannel) -> Result<SensorValue, ()> {
    let mut z_sensor_value = ZSensorValue { val1: 0, val2: 0};
    let err = unsafe {
        zephyr_sys::syscalls::any::sensor_channel_get(device as *const Device, sensor_channel as u32, (&mut z_sensor_value) as *mut ZSensorValue)
    };

    if err == 0 {
        Ok(z_sensor_value.into())
    } else {
        Err(())
    }
}

pub fn attr_set(device: &Device, sensor_channel: SensorChannel, sensor_attribute: SensorAttribute, value: SensorValue) -> Result<(), ()> {
    let z_sensor_value: ZSensorValue = value.into();
    let err = unsafe {
        zephyr_sys::syscalls::any::sensor_attr_set(device as *const Device, sensor_channel as u32, sensor_attribute as u32, (&z_sensor_value) as *const ZSensorValue)
    };

    if err == 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub fn trigger_set(device: &Device, sensor_trigger: &SensorTrigger, f: extern "C" fn(dev: &'static Device, trigger: &SensorTrigger)) -> Result<(), ()> {
    use zephyr_sys::raw::sensor_driver_api as SensorDriverApi;
    let api: Option<&SensorDriverApi> = unsafe {
        std::mem::transmute(device.api)
    };

    if let Some(api) = api {
        if let Some(trigger_set) = api.trigger_set {
            let ret = unsafe {
                let callback: extern "C" fn(dev: *const Device, trigger: *const SensorTrigger) = std::mem::transmute(f);
                (trigger_set)(device as *const Device, sensor_trigger as *const SensorTrigger, Some(callback))
            };
            if ret == 0 {
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

pub struct Sensor {
    device: &'static Device,
}

impl Sensor {
    pub fn new(device: &'static Device) -> Self {
        Sensor { device }
    }

    pub fn sample(&mut self, channel: SensorChannel) -> Result<SensorValue, ()> {
        sample_fetch_channel(self.device, channel)?;
        channel_get(self.device, channel)
    }

    pub fn set_attr(&mut self, channel: SensorChannel, attribute: SensorAttribute, value: SensorValue) -> Result<(), ()> {
        attr_set(self.device, channel, attribute, value)
    }

    pub fn set_lower_threshold(&mut self, channel: SensorChannel, value: SensorValue) -> Result<(), ()> {
        self.set_attr(channel, SensorAttribute::LowerThreshold, value)
    }

    pub fn set_upper_threshold(&mut self, channel: SensorChannel, value: SensorValue) -> Result<(), ()> {
        self.set_attr(channel, SensorAttribute::UpperThreshold, value)
    }

    pub fn enable_trigger(&mut self, trigger_type: TriggerType, channel: SensorChannel, f: extern "C" fn(dev: &'static Device, trigger: &SensorTrigger)) -> Result<(), ()> {
        let sensor_trigger = SensorTrigger {
            type_: trigger_type as u32,
            chan: channel as u32,
        };
        trigger_set(self.device, &sensor_trigger, f)
    }
}

