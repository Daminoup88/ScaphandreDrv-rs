#![cfg_attr(not(windows), allow(dead_code, unused_imports))]

#[cfg(not(windows))]
compile_error!("msr-driver-rs supports Windows only");

#[cfg(all(feature = "scaphandre", feature = "winring0"))]
compile_error!("Cannot enable both 'scaphandre' and 'winring0' features at the same time");

#[cfg(not(any(feature = "scaphandre", feature = "winring0")))]
compile_error!("At least one of 'scaphandre' or 'winring0' features must be enabled");

mod device;
mod error;
mod service;
mod util;

pub use crate::error::{Error, Result};

/// Handle to the Windows MSR driver device.
///
/// By default, this uses the Scaphandre driver. Enable the `winring0` feature to use WinRing0 instead (mutually exclusive).
pub struct MsrDriver {
    device: device::DeviceHandle,
}

impl MsrDriver {
    /// Opens the device handle. The driver must already be installed and running.
    pub fn new() -> Result<Self> {
        match device::DeviceHandle::open() {
            Ok(device) => Ok(Self { device }),
            Err(Error::NotInstalled) => {
                if service::is_installed()? {
                    return Err(Error::NotRunning);
                }
                Err(Error::NotInstalled)
            }
            Err(err) => Err(err),
        }
    }

    /// Installs the driver service and starts it (requires Administrator rights).
    pub fn install() -> Result<()> {
        service::install()
    }

    /// Returns whether the driver service exists without requiring admin rights.
    pub fn is_installed() -> Result<bool> {
        service::is_installed()
    }

    /// Returns whether the deployed driver binary is older than the one
    /// bundled in this crate build (compared by content hash).
    pub fn needs_update() -> Result<bool> {
        service::needs_update()
    }

    /// Starts an already-installed, stopped driver service (requires Administrator rights).
    pub fn start() -> Result<()> {
        service::start()
    }

    /// Closes the driver handle.
    pub fn close(&mut self) -> Result<()> {
        self.device.close()
    }

    /// Uninstalls the driver service (requires Administrator rights).
    pub fn uninstall_service() -> Result<()> {
        service::uninstall()
    }

    /// Uninstalls the driver service from a running driver instance (requires Administrator rights).
    pub fn uninstall(&mut self) -> Result<()> {
        let _ = self.close();
        service::uninstall()
    }

    /// Reads an MSR value for a given CPU index.
    pub fn read_msr(&self, msr_register: u32, cpu_index: u32) -> Result<u64> {
        self.device.read_msr(msr_register, cpu_index)
    }
}
