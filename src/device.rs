use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, HANDLE,
    INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::System::IO::DeviceIoControl;

use crate::error::{last_error, Error, Result};
use crate::util::to_utf16_z;

const DEVICE_PATH: &str = r"\\.\ScaphandreDriver";

#[repr(C)]
#[derive(Clone, Copy)]
struct DriverRequest {
    msr_register: u32,
    cpu_index: u32,
}

pub(crate) struct DeviceHandle {
    handle: HANDLE,
}

impl DeviceHandle {
    pub(crate) fn open() -> Result<Self> {
        let path_w = to_utf16_z(DEVICE_PATH);

        let handle = unsafe {
            CreateFileW(
                path_w.as_ptr(),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            let code = unsafe { GetLastError() };
            if code == ERROR_FILE_NOT_FOUND || code == ERROR_PATH_NOT_FOUND {
                return Err(Error::NotInstalled);
            }
            return Err(Error::WinApi {
                context: "CreateFileW",
                code,
            });
        }

        Ok(Self { handle })
    }

    pub(crate) fn read_msr(&self, msr_register: u32, cpu_index: u32) -> Result<u64> {
        if self.handle == INVALID_HANDLE_VALUE {
            return Err(Error::DeviceClosed);
        }

        let mut request = DriverRequest {
            msr_register,
            cpu_index,
        };

        let mut value: u64 = 0;
        let mut bytes_returned: u32 = 0;

        let ok = unsafe {
            DeviceIoControl(
                self.handle,
                0,
                &mut request as *mut DriverRequest as *mut c_void,
                size_of::<DriverRequest>() as u32,
                &mut value as *mut u64 as *mut c_void,
                size_of::<u64>() as u32,
                &mut bytes_returned,
                null_mut(),
            )
        };

        if ok == 0 {
            return Err(last_error("DeviceIoControl"));
        }

        if bytes_returned != size_of::<u64>() as u32 {
            return Err(Error::DriverProtocol {
                context: "unexpected output size",
            });
        }

        Ok(value)
    }

    pub(crate) fn close(&mut self) -> Result<()> {
        if self.handle == INVALID_HANDLE_VALUE {
            return Ok(());
        }

        let ok = unsafe { CloseHandle(self.handle) };
        self.handle = INVALID_HANDLE_VALUE;

        if ok == 0 {
            return Err(last_error("CloseHandle"));
        }

        Ok(())
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
