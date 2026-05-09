# scaphandre-driver-rs

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
- `new()` only opens the device; it does not install the driver.
- `install()` and `uninstall()` require Administrator rights.
- `is_installed()` can be used without admin rights.

## License

Apache-2.0.
