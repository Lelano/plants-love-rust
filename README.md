# plants-love-rust

This repository contains the PiGrow / "plants-love-rust" project, including a Rust firmware package in `firmware/` along with documentation, diagrams, and a bill of materials.

## Contents
- `pigrow_project_proposal.pdf` — Project proposal and high-level goals.
- `Setup/` — Setup materials and supporting documents (see directory for details).
- `SystemArch.drawio` — System architecture diagram (draw.io file).
- `System_BOM.xlsx` — Bill of materials for hardware components.

## Status
- Type: Firmware + documentation
- Code: Rust firmware present under `firmware/`
- Next steps: extend firmware logic (GPIO, sensors), add tests, deploy to Pi

## AI Usage
Some documentation and code comments were generated or assisted by AI tools (ChatGPT-5). Code logic and structure were designed and implemented by the project authors. AI was used in the ssh deploy script generation and README writing. Source library use such as RPPAL was determined by the authors, GPIO and other hardware interfacing code was authored by the project team. 


## Firmware scaffold
A minimal Rust firmware scaffold has been added in the `firmware/` folder. It contains a Cargo package you can build and run locally.

- `firmware/Cargo.toml` — package manifest for the firmware scaffold.
- `firmware/src/main.rs` — minimal `hello world` binary to replace with firmware logic.

To build and run from PowerShell:

```powershell
cd "c:\Users\adamm\Desktop\CS523 Rust Programming\plants-love-rust\firmware"
cargo build
cargo run
```

Keep docs in the repo root and code in `firmware/` to separate concerns.

## Raspberry Pi 3 A+
This project targets a Raspberry Pi 3 A+ (ARM Cortex-A53). Below are recommended ways to build and run the firmware binary for that board.

- Build directly on the Pi (easiest): install Rust via `rustup` on the Pi and `cargo build --release` in the `firmware/` folder.
- Cross-compile from another machine: use `cross` (Docker-based) or add the appropriate Rust target and a cross-linker toolchain for `armv7-unknown-linux-gnueabihf` (32-bit OS) or `aarch64-unknown-linux-gnu` (64-bit OS).

Decide which target triple matches your Pi OS:
- Raspberry Pi OS 32-bit (most common): `armv7-unknown-linux-gnueabihf`
- Raspberry Pi OS 64-bit / other 64-bit OS: `aarch64-unknown-linux-gnu`

If you need help determining your Pi's running architecture, run `uname -m` on the Pi — `armv7l` indicates the 32-bit case; `aarch64` indicates 64-bit.

## CI / Cross-compile and Deploy
A GitHub Actions workflow has been added at `.github/workflows/ci.yml` to cross-compile the `firmware` package for the common Pi targets and upload the resulting binaries as workflow artifacts.

Optional automatic deploy: the workflow contains an optional `deploy` job that will SCP the `armv7` binary to a Pi reachable at the hostname `plants-love-rust` and run it. For the deploy step to work you must add the following repository secrets in GitHub:

- `SSH_PRIVATE_KEY` — the private SSH key (PEM) for a user that can SSH to the Pi.
- `PI_USER` — the username on the Pi (commonly `pi`).

Setup on the Pi:

1. Add the corresponding public key to `/home/<pi-user>/.ssh/authorized_keys` on the Pi.

Note: For security, GitHub Actions is not recommended so that you do not expose your Pi to the public internet. Instead, connect the the Pi over local network or VPN by running the deploy step from a trusted network within which the Pi is reachable (e.g., your home network).

2. Ensure the Pi hostname `plants-love-rust` resolves on the GitHub Actions runner network (usually via your VPN or a public IP). If your Pi is behind NAT, consider a VPN or a build artifact download + manual deploy.

Security note: keep `SSH_PRIVATE_KEY` secret and give it only minimal privileges. The workflow uses `ssh` and `scp` to copy and execute the binary.

To trigger the workflow manually, go to the Actions tab and run the `CI — Cross-compile & optional deploy` workflow with `workflow_dispatch`, or push to `main`.

### Build & Deploy to Raspberry Pi (Windows, PowerShell 7)
Use the deploy script. Defaults: host `plants-love-rust`, user `user`, key `scripts/id_rsa_plants`.

```powershell
# From the REPO ROOT
# One-command: build for the Pi (auto-detect arch), upload, and run in background
pwsh -File .\scripts\deploy.ps1 -BuildLocal -Run

# Include GPIO feature (requires wiring and group permissions on the Pi)
pwsh -File .\scripts\deploy.ps1 -BuildLocal -Run -Features gpio

# Instead of running, restart a systemd service you created on the Pi
pwsh -File .\scripts\deploy.ps1 -BuildLocal -ServiceName plants-firmware

# Alternative: build on the Pi (upload source, remote cargo build), then run
pwsh -File .\scripts\deploy.ps1 -BuildOnPi -Run
pwsh -File .\scripts\deploy.ps1 -BuildOnPi -Run -Features gpio

# From the SCRIPTS FOLDER (cd .\scripts first) — drop the 'scripts/' prefix
pwsh -File .\deploy.ps1 -BuildLocal -Run -Features gpio
```

Notes:
- Requires PowerShell 7 (pwsh). Install Docker Desktop for faster cross-compiles with `cross`.
- The script uploads a binary to `~/plants-love-rust/firmware/target/release/plants_love_rust_firmware` and then runs/restarts as requested.
- The script defaults to using the SSH key at `scripts/id_rsa_plants`. Override with `-KeyPath` if needed.
- `-BuildOnPi` uploads a source archive and compiles on the Pi (first run may install Rust with rustup minimal profile).

### Binary-only deploy
If you already have a Pi-compatible binary (from cross compile or CI), you can upload and run it without building on the Pi:

```powershell
# Upload and run
pwsh -File .\scripts\deploy.ps1 -BinaryPath "C:\path\to\plants_love_rust_firmware" -Run

# Upload and restart a systemd service
pwsh -File .\scripts\deploy.ps1 -BinaryPath "C:\path\to\plants_love_rust_firmware" -ServiceName plants-firmware

# Download the latest GitHub Actions artifact and deploy
# Requires a GitHub token in $env:GITHUB_TOKEN or pass -GitHubToken
pwsh -File .\scripts\deploy.ps1 -UseLatestArtifact -Run
```

The script will detect the Pi architecture via `uname -m` and try to pick the correct binary from the artifact. Set `-BinaryName` if your artifact uses a different filename.

### Manual cross-compile (64-bit Pi OS)
If you prefer to build yourself with `cross` (Docker), then deploy:

```powershell
cargo install cross
cross build -p plants_love_rust_firmware --target aarch64-unknown-linux-gnu --release
pwsh -File .\scripts\deploy.ps1 -BinaryPath \
	".\target\aarch64-unknown-linux-gnu\release\plants_love_rust_firmware" -Run
```

### Run from the Pi GUI (TigerVNC/Desktop)
If you want a double‑click icon in the Pi’s desktop session, create a small run script and a desktop launcher on the Pi. These commands assume the default deploy paths and username `user`. Adjust if your username or `-RemoteDir` differs.

On the Pi (open a terminal in the GUI):

```bash
# 1) Create a helper script that opens in a terminal and shows output/logs
cat > /home/user/plants-love-rust/firmware/run_firmware.sh << 'EOF'
#!/usr/bin/env bash
cd /home/user/plants-love-rust/firmware
./target/release/plants_love_rust_firmware 2>&1 | tee -a run.log
read -p "Press Enter to close..."
EOF
chmod +x /home/user/plants-love-rust/firmware/run_firmware.sh

# 2) Create a desktop launcher (.desktop file)
cat > /home/user/Desktop/PlantsFirmware.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Plants Firmware
Comment=Run plants-love-rust firmware
Exec=x-terminal-emulator -e /home/user/plants-love-rust/firmware/run_firmware.sh
Path=/home/user/plants-love-rust/firmware
Terminal=false
Icon=system-run
Categories=Utility;
EOF
chmod +x /home/user/Desktop/PlantsFirmware.desktop
```

Background‑only variant (no terminal):

```bash
cat > /home/user/Desktop/PlantsFirmwareBackground.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Plants Firmware (background)
Comment=Run plants-love-rust firmware in background
Exec=/bin/bash -lc 'cd /home/user/plants-love-rust/firmware; nohup ./target/release/plants_love_rust_firmware > run.log 2>&1 &'
Path=/home/user/plants-love-rust/firmware
Terminal=false
Icon=system-run
Categories=Utility;
EOF
chmod +x /home/user/Desktop/PlantsFirmwareBackground.desktop
```

GPIO permission reminder: ensure the GUI user belongs to the `gpio` group if you use GPIO (no sudo required):

```bash
groups
sudo usermod -aG gpio user  # then re-login or reboot to apply
```


### Creating SSH keys and adding GitHub secrets
If you want the workflow to automatically deploy to your Pi (`plants-love-rust`), follow these steps locally and then add secrets in GitHub.

1) Generate a keypair on your development machine (examples included in `scripts/`):

Bash (Linux/macOS):
```bash
./scripts/generate_ssh_key.sh id_rsa_plants
```

PowerShell (Windows):
```powershell
.\scripts\generate_ssh_key.ps1 -KeyName id_rsa_plants
```

2) Copy the public key to the Pi (replace `<pi-user>`):
```bash
scp id_rsa_plants.pub <pi-user>@plants-love-rust:~/
ssh <pi-user>@plants-love-rust "mkdir -p ~/.ssh && cat ~/id_rsa_plants.pub >> ~/.ssh/authorized_keys && rm ~/id_rsa_plants.pub"
```

3) Add the private key to GitHub repository secrets:

- Open the repository on GitHub -> Settings -> Secrets -> Actions -> New repository secret.
- Secret name: `SSH_PRIVATE_KEY`
- Secret value: the full contents of `id_rsa_plants` (the private key file). Do NOT add the `.pub` file.
- Add another secret `PI_USER` with the Pi username (e.g., `pi`).

Alternative: use the `gh` CLI to create secrets (example):

```bash
# Install gh and authenticate first
gh secret set SSH_PRIVATE_KEY --body-file id_rsa_plants
gh secret set PI_USER --body "pi"
```

4) Test SSH connectivity from your machine before relying on the workflow:

```bash
ssh <pi-user>@plants-love-rust hostname && ssh <pi-user>@plants-love-rust uname -m
```

If the Pi is reachable and the above prints the hostname and architecture, the deploy step should work when the workflow runs.
## Getting started (suggested)
If you'd like to turn this into a Rust project, here are recommended initial steps (run in PowerShell):

```powershell
cd "c:\Users\adamm\Desktop\CS523 Rust Programming\plants-love-rust"
cargo init --bin    # create a new Rust binary project in this folder (or in a subfolder)
cargo build
cargo run
```

Notes:
- If you prefer to keep docs separate from code, create a `firmware/` or `src/` subdirectory and initialize the Cargo package there.
- Add a `.gitignore` for target/ and other build artifacts if you scaffold a cargo project.

## Contributing
- Open an issue or pull request describing changes.
- Suggested first tasks: add `Cargo.toml`, scaffold `src/main.rs`, add a small README section describing the runtime, and add a license.

## Contact / Author
Project files in this repo are provided by the project authors. For questions, open an issue in this repository.

---
_Automatically added README summarizing repository contents._
