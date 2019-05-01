use crate::sys;

/// Errors understood by Zathura.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PluginError {
    Unknown = 1,
    OutOfMemory,
    NotImplemented,
    InvalidArguments,
    InvalidPassword,
}

impl PluginError {
    /// Converts a raw `zathura_error_t` status code to a `PluginError`.
    ///
    /// If `error` indicates success, returns `Some(Ok(()))`, otherwise, if
    /// `error` indicates a known error code, return `Some(Err(<code>))`. If
    /// `error` is an unknown value, this function returns `None`.
    pub fn from_raw(error: sys::zathura_error_t) -> Option<Result<(), Self>> {
        Some(match error {
            0 => Ok(()),
            1 => Err(PluginError::Unknown),
            2 => Err(PluginError::OutOfMemory),
            3 => Err(PluginError::NotImplemented),
            4 => Err(PluginError::InvalidArguments),
            5 => Err(PluginError::InvalidPassword),
            _ => return None,
        })
    }
}
