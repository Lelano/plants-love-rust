# Firmware README — Raspberry Pi 3 A+

This folder contains a minimal Rust firmware scaffold for the PiGrow / `plants-love-rust` project.

Supported board: Raspberry Pi 3 A+ (ARM Cortex-A53)

Quick summary
- Build on the Pi itself (recommended for simplicity).
- Cross-compile from your development machine when you prefer faster iterations.

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
```

Option B — install Rust target and a cross linker toolchain (manual)

On your dev machine:

```bash
# Choose the appropriate target (example for 32-bit Pi OS)
rustup target add armv7-unknown-linux-gnueabihf

# Provide a cross-linker (platform specific). On Linux you can install gcc-arm-linux-gnueabihf,
# on Windows you may use WSL or a cross toolchain.

# Optionally add a `.cargo/config.toml` to point the linker to the cross-gcc:
# [target.armv7-unknown-linux-gnueabihf]
# linker = "arm-linux-gnueabihf-gcc"

cargo build --target armv7-unknown-linux-gnueabihf --release
```

3) Copy the binary to the Pi and run

```bash
scp target/armv7-unknown-linux-gnueabihf/release/plants_love_rust_firmware pi@<pi-ip>:/home/pi/
ssh pi@<pi-ip>
chmod +x plants_love_rust_firmware
./plants_love_rust_firmware
```

Notes
- If you build directly on the Pi, you don't need cross tools; `cargo build --release` will produce a runnable binary for the Pi's OS.
- If you plan to use hardware interfaces (GPIO, I2C, SPI), add the appropriate crates and run with the required OS permissions or system services.

GPIO example
- A gated GPIO example is included and uses the `rppal` crate. It is disabled by default so building on non-Linux hosts (Windows/macOS) succeeds.

To build the GPIO example (on the Pi or when you have a suitable cross-toolchain):

```bash
# Build on the Pi (recommended):
cd firmware
cargo build --release --features gpio

# Or cross-compile (example for 32-bit Pi OS):
rustup target add armv7-unknown-linux-gnueabihf
# install an appropriate cross-linker such as `gcc-arm-linux-gnueabihf` and ensure the linker name
# matches `arm-linux-gnueabihf-gcc` (or update `firmware/.cargo/config.toml` accordingly)
cargo build --target armv7-unknown-linux-gnueabihf --release --features gpio
```

The example will blink BCM pin 17 five times. Modify `firmware/src/gpio_example.rs` to change the pin or behavior.

If you want, I can add wiring notes for common sensors (DHT22, soil moisture) and a simple crate list.
