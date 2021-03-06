#![feature(arbitrary_enum_discriminant)]
#![feature(const_fn_transmute)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(const_fn)]
#![feature(const_ptr_offset)]
extern crate zephyr_sys;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, write};

#[cfg(feature = "bluetooth")]
pub mod bluetooth;
pub mod gpio;
pub mod network;
pub mod sensor;

/// Trait for a context in which an error can occur.
pub trait Context: Debug {
    fn name(&self) -> &'static str;
}

/// List of the error numbers used in the Zephyr APIs.
///
/// Zephyr also uses negative numbers for error numbers. That is why some error numbers occur positive
/// and negative. Negative error numbers are prefixed with a capital `N`.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorNumber {
    Permission = 1,
    NotImplemented = 88,
    NotConnected = 128,
    Other(i32),
}

impl From<i32> for ErrorNumber {
    fn from(errno: i32) -> Self {
        match errno {
            1 => ErrorNumber::Permission,
            88 | -88 => ErrorNumber::NotImplemented,
            128 | -128 => ErrorNumber::NotConnected,
            errno => ErrorNumber::Other(errno.abs()),
        }
    }
}

impl Display for ErrorNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorNumber::Permission => {
                write!(f, "1: Not owner")
            }
            ErrorNumber::Other(errno) => {
                write!(f, "Unknown error number: {}", errno)
            }
            ErrorNumber::NotImplemented => {
                write!(f, "88: Function not implemented")
            }
            ErrorNumber::NotConnected => {
                write!(f, "128: Not connected")
            }
        }
    }
}

/// Error that might occur in the Zephyr API. Such errors might also occur in the wrapper implementations.
///
/// Errors generated by the wrapper functions will never have a negative error number.
#[derive(Debug)]
pub struct ZephyrError {
    errno: ErrorNumber,
    context: Option<&'static dyn Context>,
}

impl ZephyrError {
    /// Create a new error
    pub fn new(errno: ErrorNumber) -> Self {
        Self {
            errno,
            context: None,
        }
    }

    /// Create a new error with the given context
    pub fn new_with_context(errno: ErrorNumber, context: &'static dyn Context) -> Self {
        Self {
            errno,
            context: Some(context),
        }
    }

    /// Convert `errno` into a [ErrorNumber] variant and construct a new error
    pub fn from_errno(errno: i32) -> Self {
        Self {
            errno: errno.into(),
            context: None,
        }
    }

    /// Convert `errno` into a [ErrorNumber] variant and construct a new error with the given context
    pub fn from_errno_with_context(errno: i32, context: &'static dyn Context) -> Self {
        Self {
            errno: errno.into(),
            context: Some(context),
        }
    }

    pub fn number(&self) -> ErrorNumber {
        self.errno
    }
}

impl Display for ZephyrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(context) = self.context {
            write!(f, "[{}]: ", context.name())?;
        }
        write!(f, "{}", self.errno)
    }
}

impl Error for ZephyrError {}

pub type ZephyrResult<T> = Result<T, ZephyrError>;
