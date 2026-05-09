#![cfg(windows)]

use std::thread;
use std::time::{Duration, Instant};

use scaphandre_driver_rs::ScaphandreDriver;

const MSR_RAPL_POWER_UNIT: u32 = 0x0000_0606;
const MSR_PKG_ENERGY_STATUS: u32 = 0x0000_0611;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !ScaphandreDriver::is_installed()? {
        eprintln!("Driver not installed. Run ScaphandreDriver::install() from an elevated terminal.");
        return Ok(());
    }

    let driver = ScaphandreDriver::new()?;

    let unit_raw = driver.read_msr(MSR_RAPL_POWER_UNIT, 0)?;
    let energy_unit_exponent = ((unit_raw >> 8) & 0x1f) as u32;
    let joules_per_lsb = 1.0f64 / (1u64 << energy_unit_exponent) as f64;

    let mut previous_energy = (driver.read_msr(MSR_PKG_ENERGY_STATUS, 0)? & 0xffff_ffff) as u32;
    let mut previous_time = Instant::now();

    loop {
        thread::sleep(Duration::from_secs(1));

        let now = Instant::now();
        let current_energy = (driver.read_msr(MSR_PKG_ENERGY_STATUS, 0)? & 0xffff_ffff) as u32;

        let delta_raw = current_energy.wrapping_sub(previous_energy) as f64;
        let elapsed_s = now.duration_since(previous_time).as_secs_f64();

        if elapsed_s > 0.0 {
            let delta_joules = delta_raw * joules_per_lsb;
            let power_watts = delta_joules / elapsed_s;
            println!(
                "pkg_power={:.3} W delta_j={:.6} elapsed_s={:.3}",
                power_watts, delta_joules, elapsed_s
            );
        }

        previous_energy = current_energy;
        previous_time = now;
    }
}
