# msr-driver-rs [![](https://img.shields.io/crates/v/msr-driver-rs.svg)](https://crates.io/crates/msr-driver-rs)

Minimal Rust wrapper around Windows MSR drivers (Scaphandre or WinRing0).

## Features

- `scaphandre` (default): Uses the Scaphandre RAPL driver
- `winring0`: Uses the WinRing0 driver

**Note:** Only one feature can be enabled at a time.

### Using Scaphandre

To use the Scaphandre driver, execute the following:

```bash
cargo add msr-driver-rs
```

### Using WinRing0

To use WinRing0 instead of Scaphandre, execute the following:

```bash
cargo add msr-driver-rs --no-default-features --features winring0
```

## Usage

```rust
use msr_driver_rs::MsrDriver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = MsrDriver::new()?;
    let msr_pkg_energy_status = 0x0000_0611;
    let value = driver.read_msr(msr_pkg_energy_status, 0)?;
    println!("MSR value: {value:#x}");
    Ok(())
}
```

## Admin operations

```rust
MsrDriver::install()?;
MsrDriver::uninstall()?;
```

## Notes

- Windows only.
- `new()` opens the device handle. It does not install the driver.
- `install()`, `uninstall()` and `start()` require Administrator privileges.
- `is_installed()` can be used without admin rights.

## Example

A working example is included:

```
cargo run --example wattseal_like
```

It polls the energy counter, calculates the joules consumed, and average power consumption, such as what [WattSeal](https://github.com/Daminoup88/WattSeal) does.

## License

Apache-2.0.
