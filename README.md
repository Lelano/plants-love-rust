# plants-love-rust

This repository holds planning and documentation files for the "plants-love-rust" / PiGrow project. There is currently no Rust source code in this repository — it contains project artifacts, diagrams, and a bill of materials.

## Contents
- `pigrow_project_proposal.pdf` — Project proposal and high-level goals.
- `Setup/` — Setup materials and supporting documents (see directory for details).
- `SystemArch.drawio` — System architecture diagram (draw.io file).
- `System_BOM.xlsx` — Bill of materials for hardware components.

## Status
- Type: Documentation / planning
- Code: No Rust/Cargo project present yet
- Next steps: scaffold Rust project, implement firmware/control code, and add CI/tests

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
2. Ensure the Pi hostname `plants-love-rust` resolves on the GitHub Actions runner network (usually via your VPN or a public IP). If your Pi is behind NAT, consider a VPN or a build artifact download + manual deploy.

Security note: keep `SSH_PRIVATE_KEY` secret and give it only minimal privileges. The workflow uses `ssh` and `scp` to copy and execute the binary.

To trigger the workflow manually, go to the Actions tab and run the `CI — Cross-compile & optional deploy` workflow with `workflow_dispatch`, or push to `main`.

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
