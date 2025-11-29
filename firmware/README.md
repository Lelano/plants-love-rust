# Firmware README — Raspberry Pi 3 A+

This folder contains a minimal Rust firmware scaffold for the PiGrow / `plants-love-rust` project.

Supported board: Raspberry Pi 3 A+ (ARM Cortex-A53)

Quick summary
- Build on the Pi itself (recommended for simplicity).
- Cross-compile from your development machine when you prefer faster iterations.

Build & deploy from Windows (PowerShell 7)

```powershell
# From the REPO ROOT
# Build for the Pi (auto-detect arch), upload, and run in background
pwsh -File .\scripts\deploy.ps1 -BuildLocal -Run

# Include GPIO feature
pwsh -File .\scripts\deploy.ps1 -BuildLocal -Run -Features gpio

# Restart a systemd service after upload
pwsh -File .\scripts\deploy.ps1 -BuildLocal -ServiceName plants-firmware

# From the SCRIPTS FOLDER (cd .\scripts first)
pwsh -File .\deploy.ps1 -BuildLocal -Run -Features gpio
```

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

GPIO example
- A gated GPIO example is included and uses the `rppal` crate. It is disabled by default so building on non-Linux hosts (Windows/macOS) succeeds.

To build the GPIO example (on the Pi or when you have a suitable cross-toolchain):

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

The example blinks BCM pin 17 continuously and logs `GPIO17 -> HIGH/LOW`. Modify `firmware/src/gpio_example.rs` to change the pin or behavior.

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
