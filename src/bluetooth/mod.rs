use crate::Context;
use std::fmt::{Debug, Formatter};

pub mod api;
pub mod connection;
pub mod data;
pub mod gatt;
pub mod uuid;

pub(self) struct BluetoothContext {}
pub(self) static CONTEXT: BluetoothContext = BluetoothContext {};

impl Debug for BluetoothContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "bluetooth")
    }
}

impl Context for BluetoothContext {
    fn name(&self) -> &'static str {
        "bluetooth"
    }
}

pub mod le;
