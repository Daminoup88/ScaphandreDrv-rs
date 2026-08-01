# scaphandre-driver-rs [![](https://img.shields.io/crates/v/scaphandre-driver-rs.svg)](https://crates.io/crates/scaphandre-driver-rs)

Minimal Rust wrapper around the Scaphandre Windows RAPL driver.

## Usage

```rust
use scaphandre_driver_rs::ScaphandreDriver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = ScaphandreDriver::new()?;
    let msr_pkg_energy_status = 0x0000_0611;
    let value = driver.read_msr(msr_pkg_energy_status, 0)?;
    println!("MSR value: {value:#x}");
    Ok(())
}
```

## Admin operations

```rust
ScaphandreDriver::install()?;
ScaphandreDriver::uninstall()?;
```

## Notes

- Windows only.
- `new()` opens the device handle. It does not install the driver.
- `install()` and `uninstall()` require Administrator privileges.
- `is_installed()` can be used without admin rights.

## Example

A working example is included:

```
cargo run --example wattseal_like
```

It polls the energy counter, calculates the joules consumed, and average power consumption, such as what [WattSeal](https://github.com/Daminoup88/WattSeal) does.

## License

Apache-2.0.
