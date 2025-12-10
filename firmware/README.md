# Firmware README — Raspberry Pi 3 A+

This folder contains Rust firmware for the PiGrow / `plants-love-rust` project with GPIO control, scheduling, and **soil moisture sensing via ADS1115 ADC**.

Supported board: Raspberry Pi 3 A+ (ARM Cortex-A53)

Features:
- Interval-based and schedule-based GPIO control
- Real-time soil moisture monitoring via I2C ADS1115 16-bit ADC
- Interactive terminal UI with live sensor display and calibration
- Persistent configuration with TOML

Features:
- Interval-based and schedule-based GPIO control
- Real-time soil moisture monitoring via I2C ADS1115 16-bit ADC
- Interactive terminal UI with live sensor display and calibration
- Persistent configuration with TOML

## Quick summary
- Build on the Pi itself (recommended for simplicity).
- Cross-compile from your development machine when you prefer faster iterations.

## Hardware Setup

### Enable I2C
Before running, enable I2C on the Raspberry Pi:
```bash
sudo raspi-config
# Interface Options → I2C → Enable
sudo reboot
```

Verify the ADS1115 is detected:
```bash
i2cdetect -y 1
# Should show device at address 0x48
```

### Wiring
See the main README for complete wiring diagram. Quick reference:
- **ADS1115**: VDD→3.3V, GND→GND, SCL→GPIO3, SDA→GPIO2, ADDR→GND
- **Moisture Sensor**: VCC→3.3V/5V, GND→GND, AOUT→ADS1115 A3

## Build & deploy from Windows (PowerShell 7)

```powershell
# From the REPO ROOT
# Build for the Pi (auto-detect arch), upload, and run in background
# Note that docker must be running on your local machine for cross-compilation to work.
pwsh -File .\scripts\deploy.ps1 -BuildLocal -Run

# Include GPIO feature
pwsh -File .\scripts\deploy.ps1 -BuildLocal -Run -Features gpio

# Restart a systemd service after upload
pwsh -File .\scripts\deploy.ps1 -BuildLocal -ServiceName plants-firmware

# From the SCRIPTS FOLDER (cd .\scripts first)
pwsh -File .\deploy.ps1 -BuildLocal -Run -Features gpio
```

## UI
- The firmware provides a terminal UI. Run it in a terminal on the Pi.
- **Controls:**
  - `q`/`Esc`: Quit
  - `b`: Toggle GPIO blink
  - `+`/`-`: Adjust interval (ms)
  - `d`: Calibrate dry value (place sensor in dry air/soil, then press)
  - `w`: Calibrate wet value (place sensor in water/saturated soil, then press)
- **Display:**
  - GPIO pin status and interval
  - Live moisture sensor readings: raw ADC value, voltage, moisture %
  - Calibration status (dry/wet values)

## Sensor Calibration
1. Run the firmware: `./plants_love_rust_firmware`
2. With sensor in **dry** conditions, press `d` to capture dry value
3. With sensor in **wet** conditions (water or saturated soil), press `w`
4. Values are saved to `~/.config/plants-love-rust/config.toml`
5. Moisture % will now display based on calibration

Schedule (optional)
- You can define a GPIO schedule in `~/.config/plants-love-rust/config.toml`.
- Example:

```toml
# Pin used for schedule controller
schedule_pin = 27

[schedule]
Monday = [[0, 900]]
Tuesday = [[0, 900]]
Wednesday = [[0, 900]]
Thursday = [[0, 900]]
Friday = [[0, 900]]
Saturday = [[0, 900]]
Sunday = [[0, 900]]
```

Validation rules
- Times are HHMM: `0..=2359` with minutes `< 60`.
- Each range must have `start < end`.
- Overlapping or adjacent ranges are merged automatically.
- Invalid entries are ignored and logged at startup.

- If a schedule is present, the firmware starts a scheduler on `schedule_pin` and also runs the interactive interval controller on `gpio_pin`.

1) Build on the Raspberry Pi (recommended)

- On the Pi (Raspberry Pi OS), install Rust and build:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
cd firmware
cargo build --release
```

The release binary will be in `target/release/plants_love_rust_firmware` (or with the target triple subfolder if you added explicit targets).

2) Cross-compile from Windows/macOS/Linux

Option A — use `cross` (easiest, uses Docker):

```bash
cargo install cross
# Example: build for 32-bit Raspberry Pi OS
cross build --target armv7-unknown-linux-gnueabihf --release
# Example: build for 64-bit Raspberry Pi OS
cross build -p plants_love_rust_firmware --target aarch64-unknown-linux-gnu --release
```

Option B — install Rust target and a cross linker toolchain (manual)

On your dev machine:

```bash
# Choose the appropriate target
# 32-bit Pi OS
rustup target add armv7-unknown-linux-gnueabihf
# 64-bit Pi OS
rustup target add aarch64-unknown-linux-gnu

# Provide a cross-linker (platform specific). On Linux you can install gcc-arm-linux-gnueabihf,
# on Windows you may use WSL or a cross toolchain.

# Optionally add a `.cargo/config.toml` to point the linker to the cross-gcc:
# [target.armv7-unknown-linux-gnueabihf]
# linker = "arm-linux-gnueabihf-gcc"

# Build using cargo (example 32-bit). For 64-bit, replace the target triple accordingly.
cargo build --target armv7-unknown-linux-gnueabihf --release
```

3) Copy the binary to the Pi and run

```bash
scp target/<triple>/release/plants_love_rust_firmware user@plants-love-rust:/home/user/
ssh user@plants-love-rust
chmod +x plants_love_rust_firmware
./plants_love_rust_firmware
```

Notes
- If you build directly on the Pi, you don't need cross tools; `cargo build --release` will produce a runnable binary for the Pi's OS.
- If you plan to use hardware interfaces (GPIO, I2C, SPI), add the appropriate crates and run with the required OS permissions or system services.

## GPIO and Sensor Features
- GPIO control is gated behind the `gpio` feature using the `rppal` crate. It is disabled by default so building on non-Linux hosts (Windows/macOS) succeeds.
- The **analog module** provides I2C communication with the ADS1115 ADC for reading the capacitive moisture sensor on channel A3.

To build with GPIO and sensor support (on the Pi or when you have a suitable cross-toolchain):

```bash
# Build on the Pi (recommended):
cd firmware
cargo build --release --features gpio

# Or cross-compile (examples):
# 32-bit Pi OS
rustup target add armv7-unknown-linux-gnueabihf
cargo build --target armv7-unknown-linux-gnueabihf --release --features gpio
# 64-bit Pi OS
rustup target add aarch64-unknown-linux-gnu
cargo build --target aarch64-unknown-linux-gnu --release --features gpio
```

The firmware uses BCM pin 17 for interval control and pin 27 for optional scheduling. Modify settings in the config file or via the UI.

Deploy script default paths on the Pi
- Project root: `/home/user/plants-love-rust`
- Firmware dir: `/home/user/plants-love-rust/firmware`
- Binary: `/home/user/plants-love-rust/firmware/target/release/plants_love_rust_firmware`
- Log (when using `-Run`): `/home/user/plants-love-rust/firmware/run.log`

GPIO permissions
- Ensure the `user` is in the `gpio` group to access `/dev/gpiomem` without sudo:
```bash
groups
# if missing:
sudo usermod -aG gpio user
# then re-login or reboot
```
