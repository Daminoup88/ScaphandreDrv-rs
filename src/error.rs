use std::fmt;

use windows_sys::Win32::Foundation::GetLastError;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    WinApi {
        context: &'static str,
        code: u32,
    },
    DriverProtocol {
        context: &'static str,
    },
    NotInstalled,
    NotRunning,
    DeviceClosed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::WinApi { context, code } => {
                write!(f, "Windows API error in {context} (code {code})")
            }
            Self::DriverProtocol { context } => write!(f, "Driver protocol error: {context}"),
            Self::NotInstalled => write!(f, "Driver is not installed"),
            Self::NotRunning => write!(f, "Driver is installed but not running"),
            Self::DeviceClosed => write!(f, "Driver handle is closed"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn last_error(context: &'static str) -> Error {
    Error::WinApi {
        context,
        code: unsafe { GetLastError() },
    }
}
