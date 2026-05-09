use std::fs;
use std::path::PathBuf;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{
    GetLastError, ERROR_ALREADY_EXISTS, ERROR_SERVICE_ALREADY_RUNNING, ERROR_SERVICE_DOES_NOT_EXIST,
    ERROR_SERVICE_EXISTS,
};
use windows_sys::Win32::System::Services::{
    CloseServiceHandle, ControlService, CreateServiceW, DeleteService, OpenSCManagerW,
    OpenServiceW, StartServiceW, SC_HANDLE, SC_MANAGER_ALL_ACCESS, SC_MANAGER_CONNECT,
    SERVICE_ALL_ACCESS, SERVICE_CONTROL_STOP, SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL,
    SERVICE_KERNEL_DRIVER, SERVICE_QUERY_STATUS, SERVICE_STATUS,
};

use crate::error::{last_error, Error, Result};
use crate::util::to_utf16_z;

const DRIVER_BYTES: &[u8] = include_bytes!("../ScaphandreDrv.sys");
const SERVICE_NAME: &str = "ScaphandreDrv";
const SERVICE_DISPLAY_NAME: &str = "Scaphandre Driver Service";

pub(crate) fn is_installed() -> Result<bool> {
    let manager = open_service_manager(SC_MANAGER_CONNECT)?;

    let service_name_w = to_utf16_z(SERVICE_NAME);
    let service = unsafe { OpenServiceW(manager, service_name_w.as_ptr(), SERVICE_QUERY_STATUS) };

    if service.is_null() {
        let code = unsafe { GetLastError() };
        unsafe { CloseServiceHandle(manager) };
        if code == ERROR_SERVICE_DOES_NOT_EXIST {
            return Ok(false);
        }
        return Err(Error::WinApi {
            context: "OpenServiceW",
            code,
        });
    }

    unsafe {
        CloseServiceHandle(service);
        CloseServiceHandle(manager);
    }

    Ok(true)
}

pub(crate) fn install() -> Result<()> {
    let driver_path = deploy_driver_binary()?;
    let manager = open_service_manager(SC_MANAGER_ALL_ACCESS)?;

    let service_name_w = to_utf16_z(SERVICE_NAME);
    let display_name_w = to_utf16_z(SERVICE_DISPLAY_NAME);
    let driver_path_w = to_utf16_z(driver_path.to_string_lossy().as_ref());

    let service = unsafe {
        CreateServiceW(
            manager,
            service_name_w.as_ptr(),
            display_name_w.as_ptr(),
            SERVICE_ALL_ACCESS,
            SERVICE_KERNEL_DRIVER,
            SERVICE_DEMAND_START,
            SERVICE_ERROR_NORMAL,
            driver_path_w.as_ptr(),
            null(),
            null_mut(),
            null(),
            null(),
            null(),
        )
    };

    let service_handle = if service.is_null() {
        let code = unsafe { GetLastError() };
        if code == ERROR_SERVICE_EXISTS || code == ERROR_ALREADY_EXISTS {
            let existing = unsafe { OpenServiceW(manager, service_name_w.as_ptr(), SERVICE_ALL_ACCESS) };
            if existing.is_null() {
                unsafe { CloseServiceHandle(manager) };
                return Err(last_error("OpenServiceW"));
            }
            existing
        } else {
            unsafe { CloseServiceHandle(manager) };
            return Err(Error::WinApi {
                context: "CreateServiceW",
                code,
            });
        }
    } else {
        service
    };

    let started = unsafe { StartServiceW(service_handle, 0, null()) };
    if started == 0 {
        let code = unsafe { GetLastError() };
        if code != ERROR_SERVICE_ALREADY_RUNNING {
            unsafe {
                CloseServiceHandle(service_handle);
                CloseServiceHandle(manager);
            }
            return Err(Error::WinApi {
                context: "StartServiceW",
                code,
            });
        }
    }

    unsafe {
        CloseServiceHandle(service_handle);
        CloseServiceHandle(manager);
    }

    Ok(())
}

pub(crate) fn uninstall() -> Result<()> {
    let manager = open_service_manager(SC_MANAGER_ALL_ACCESS)?;
    let service_name_w = to_utf16_z(SERVICE_NAME);
    let service = unsafe { OpenServiceW(manager, service_name_w.as_ptr(), SERVICE_ALL_ACCESS) };

    if service.is_null() {
        let code = unsafe { GetLastError() };
        unsafe { CloseServiceHandle(manager) };
        if code == ERROR_SERVICE_DOES_NOT_EXIST {
            remove_driver_binary();
            return Ok(());
        }
        return Err(Error::WinApi {
            context: "OpenServiceW",
            code,
        });
    }

    let mut status = SERVICE_STATUS {
        dwServiceType: 0,
        dwCurrentState: 0,
        dwControlsAccepted: 0,
        dwWin32ExitCode: 0,
        dwServiceSpecificExitCode: 0,
        dwCheckPoint: 0,
        dwWaitHint: 0,
    };

    let _ = unsafe { ControlService(service, SERVICE_CONTROL_STOP, &mut status) };

    let deleted = unsafe { DeleteService(service) };
    if deleted == 0 {
        let code = unsafe { GetLastError() };
        unsafe {
            CloseServiceHandle(service);
            CloseServiceHandle(manager);
        }
        return Err(Error::WinApi {
            context: "DeleteService",
            code,
        });
    }

    unsafe {
        CloseServiceHandle(service);
        CloseServiceHandle(manager);
    }

    remove_driver_binary();

    Ok(())
}

fn open_service_manager(access: u32) -> Result<SC_HANDLE> {
    let manager = unsafe { OpenSCManagerW(null(), null(), access) };
    if manager.is_null() {
        return Err(last_error("OpenSCManagerW"));
    }
    Ok(manager)
}

fn deploy_driver_binary() -> Result<PathBuf> {
    let path = driver_binary_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        fs::write(&path, DRIVER_BYTES)?;
    }

    Ok(path)
}

fn remove_driver_binary() {
    let path = driver_binary_path();
    let _ = fs::remove_file(path);
}

fn driver_binary_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("scaphandre-driver-rs");
    path.push("ScaphandreDrv.sys");
    path
}
