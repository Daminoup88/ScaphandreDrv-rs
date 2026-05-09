#![cfg(windows)]

use scaphandre_driver_rs::ScaphandreDriver;

const MSR_RAPL_POWER_UNIT: u32 = 0x0000_0606;

#[test]
#[ignore]
fn smoke_read_msr() {
    ScaphandreDriver::install().expect("driver install failed");
    let driver = ScaphandreDriver::new().expect("driver init failed");
    let _ = driver
        .read_msr(MSR_RAPL_POWER_UNIT, 0)
        .expect("read_msr failed");
}
