<#
Deploy plants-love-rust firmware to a Raspberry Pi from PowerShell 7 (pwsh).

Quick usage (binary-only upload):
  pwsh -File ./scripts/deploy.ps1 -BinaryPath "./target/aarch64-unknown-linux-gnu/release/plants_love_rust_firmware" -Run

Quick usage (build locally with cross + Docker, then upload & run):
  pwsh -File ./scripts/deploy.ps1 -BuildLocal -Run

Defaults:
  -PiHost plants-love-rust, -PiUser user, -KeyPath scripts/id_rsa_plants
  Upload path on Pi: ~/plants-love-rust/firmware/target/release

Requires:
  - PowerShell 7+ (pwsh)
  - OpenSSH client (ssh/scp) in PATH
  - For -BuildLocal: cargo; recommended: cross + Docker Desktop running
#>

param(
  [string]$PiHost = "plants-love-rust",
  [string]$PiUser = "user",
  [string]$KeyPath = "$(Join-Path $PSScriptRoot 'id_rsa_plants')",
  [string]$RemoteDir = "~/plants-love-rust",

  [switch]$Run,
  [string]$ServiceName,

  # Provide a prebuilt binary to upload (skips building on the Pi)
  [string]$BinaryPath,

  # Build locally for the Pi (auto-detects arch via SSH) and then upload
  [switch]$BuildLocal,

  # Build on the Pi (upload source archive, remote cargo build)
  [switch]$BuildOnPi,

  # Optional features when building locally (e.g. "gpio")
  [string]$Features
)

$ErrorActionPreference = 'Stop'
if (-not $PSVersionTable.PSVersion -or $PSVersionTable.PSVersion.Major -lt 7) {
  throw "This script requires PowerShell 7+. Please run with 'pwsh'."
}

function Info($m){ Write-Host "[INFO] $m" -ForegroundColor Cyan }
function Warn($m){ Write-Host "[WARN] $m" -ForegroundColor Yellow }
function Err ($m){ Write-Host "[ERR ] $m" -ForegroundColor Red }

function Assert-Cmd($name){ if (-not (Get-Command $name -ErrorAction SilentlyContinue)) { Err "Missing command: $name"; exit 2 } }
Assert-Cmd ssh; Assert-Cmd scp

# Resolve repo root (folder containing root Cargo.toml)
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..') | Select-Object -ExpandProperty Path

if (-not (Test-Path (Join-Path $repoRoot 'Cargo.toml'))) {
  Err "Could not find Cargo.toml at repo root: $repoRoot"
  exit 1
}

if (-not (Test-Path $KeyPath)) { Err "SSH key not found at: $KeyPath"; exit 3 }

# SSH base args
$sshArgs = @('-i', $KeyPath, '-o', 'StrictHostKeyChecking=no', '-o', 'UserKnownHostsFile=/dev/null', "$PiUser@$PiHost")

# Test SSH
Info "Testing SSH connectivity to $PiUser@$PiHost ..."
& ssh @sshArgs 'echo ok' | Out-Null

# Helper: map uname -m to Rust target
function Get-TargetTriple($arch){
  switch -regex ($arch) {
    'aarch64' { 'aarch64-unknown-linux-gnu' ; break }
    'armv7'   { 'armv7-unknown-linux-gnueabihf' ; break }
    'armv6'   { 'arm-unknown-linux-gnueabihf' ; break }
    default   { $null }
  }
}

# If requested, build locally for the Pi and set BinaryPath
if ($BuildLocal.IsPresent -and -not $BinaryPath) {
  Assert-Cmd cargo
  Info "Detecting Pi architecture via uname -m ..."
  $piArch = (& ssh @sshArgs 'uname -m').Trim()
  if (-not $piArch) { Err 'Failed to detect Pi arch'; exit 7 }
  $target = Get-TargetTriple $piArch
  if (-not $target) { Err "Unsupported Pi arch '$piArch'"; exit 7 }
  Info "Pi arch: $piArch -> target: $target"

  $useCross = $false
  $haveCross = Get-Command cross -ErrorAction SilentlyContinue
  $haveDocker = Get-Command docker -ErrorAction SilentlyContinue
  if (-not $haveCross -and $haveDocker) {
    Info "Installing cross (first run only) ..."
    & cargo install cross
    $haveCross = Get-Command cross -ErrorAction SilentlyContinue
  }
  if ($haveCross -and $haveDocker) { $useCross = $true } else { Warn "Using cargo build (no cross/docker) — ensure toolchain for $target exists" }

  Push-Location $repoRoot
  try {
    $feat = ($Features -and $Features.Trim()) ? @('--features', $Features.Trim()) : @()
    if ($useCross) {
      Info "Running: cross build -p plants_love_rust_firmware --target $target --release $($feat -join ' ')"
      & cross build -p plants_love_rust_firmware --target $target --release @feat
    } else {
      # Ensure target added
      if (Get-Command rustup -ErrorAction SilentlyContinue) {
        & rustup target add $target | Out-Null
      }
      Info "Running: cargo build -p plants_love_rust_firmware --target $target --release $($feat -join ' ')"
      & cargo build -p plants_love_rust_firmware --target $target --release @feat
    }
  }
  finally { Pop-Location }

  if ($LASTEXITCODE -ne 0) { Err "Local build failed (exit $LASTEXITCODE)"; exit $LASTEXITCODE }

  $BinaryPath = Join-Path (Join-Path (Join-Path $repoRoot 'target') $target) 'release/plants_love_rust_firmware'
  if (-not (Test-Path $BinaryPath)) { Err "Built binary not found: $BinaryPath"; exit 8 }
  Info "Using locally built binary: $BinaryPath"
}

if ($BuildOnPi.IsPresent -and -not $BinaryPath -and -not $BuildLocal.IsPresent) {
  # Create source archive and build remotely on the Pi
  Info "Creating source archive for remote build ..."
  $timestamp = Get-Date -Format 'yyyyMMddHHmmss'
  $archiveBase = "plants-$timestamp"
  $tarPath = Join-Path $env:TEMP "$archiveBase.tar.gz"
  $zipPath = Join-Path $env:TEMP "$archiveBase.zip"
  $tarAvailable = $false
  if (Get-Command tar -ErrorAction SilentlyContinue) { $tarAvailable = $true }

  Push-Location $repoRoot
  try {
    if ($tarAvailable) {
      $exclude = @('--exclude=.git','--exclude=target','--exclude=firmware/target')
      & tar -czf $tarPath $exclude .
      Info "Created archive: $tarPath"
    } else {
      Warn 'tar not found; falling back to Zip archive (slower, larger)'
      if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
      $items = Get-ChildItem -Force | Where-Object { $_.Name -notin '.git','target' }
      Compress-Archive -Path $items -DestinationPath $zipPath -Force
      Info "Created archive: $zipPath"
    }
  }
  finally { Pop-Location }

  $localArchive = if ($tarAvailable) { $tarPath } else { $zipPath }
  $remoteArchive = "/tmp/$([System.IO.Path]::GetFileName($localArchive))"
  Info "Uploading archive to $remoteArchive ..."
  $destArc = "$PiUser@$PiHost`:$remoteArchive"
  & scp -i $KeyPath -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null $localArchive $destArc | Out-Null

  # Feature args
  $featureArgs = ''
  if ($Features -and $Features.Trim() -ne '') {
    $featureList = ($Features -split '[,\s]+' | Where-Object { $_ -ne '' }) -join ','
    $featureArgs = " --features $featureList"
  }

  # Compose remote build script
  $extractCmd = if ($tarAvailable) {
    ("mkdir -p {0}; tar -xzf {1} -C {0}; rm -f {1}" -f $RemoteDir, $remoteArchive)
  } else {
    ("mkdir -p {0}; if unzip -oq {1} -d {0}; then :; else bsdtar -xf {1} -C {0}; fi; rm -f {1}" -f $RemoteDir, $remoteArchive)
  }

  $remote = @"
set -e
$extractCmd
if ! command -v cargo >/dev/null 2>&1; then
  echo Installing Rust (rustup minimal profile) ...
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  . "\$HOME/.cargo/env"
fi
cd $RemoteDir
if [ -f firmware/Cargo.toml ]; then cd firmware; fi
echo Building on Pi ...
cargo build --release$featureArgs
BIN=plants_love_rust_firmware
if [ ! -f "target/release/\$BIN" ]; then echo Build succeeded but binary not found; exit 11; fi
"@

  $remoteUnix = $remote -replace "`r", ""
  & ssh @sshArgs $remoteUnix
  if ($LASTEXITCODE -ne 0) { Err "Remote build failed (exit $LASTEXITCODE)"; exit $LASTEXITCODE }
}

if (-not $BinaryPath -and -not $BuildOnPi.IsPresent -and -not $BuildLocal.IsPresent) {
  Err 'No -BinaryPath provided and neither -BuildLocal nor -BuildOnPi set. Provide a prebuilt binary or choose a build mode.'
  exit 9
}

if (-not (Test-Path $BinaryPath)) { Err "BinaryPath not found: $BinaryPath"; exit 5 }

# Upload binary and run/service (skipped if we built on Pi; binary already present)
$remoteBinDir  = "$RemoteDir/firmware/target/release"
$remoteBinPath = "$remoteBinDir/plants_love_rust_firmware"

if ($BinaryPath) {
  Info "Preparing remote directory $remoteBinDir ..."
  & ssh @sshArgs "mkdir -p $remoteBinDir"
  if ($LASTEXITCODE -ne 0) { Err 'Failed to prepare remote dir'; exit $LASTEXITCODE }

  Info "Uploading binary to $remoteBinPath ..."
  $dest = "$PiUser@$PiHost`:$remoteBinPath"
  & scp -i $KeyPath -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null `
    "$BinaryPath" $dest | Out-Null

  Info 'Setting execute permission ...'
  & ssh @sshArgs "chmod +x $remoteBinPath"
  if ($LASTEXITCODE -ne 0) { Err 'chmod failed'; exit $LASTEXITCODE }
}

if ($ServiceName) {
  Info "Restarting service $ServiceName ..."
  $remote = @'
set -e
cd $REMOTE_DIR/firmware
sudo systemctl daemon-reload || true
sudo systemctl restart SERVICE_NAME
sudo systemctl status --no-pager --lines=50 SERVICE_NAME || true
'@
  $remote = $remote.Replace('$REMOTE_DIR', $RemoteDir).Replace('SERVICE_NAME', $ServiceName)
  # Normalize line endings to LF for bash on the Pi
  $remoteUnix = $remote -replace "`r", ""
  & ssh @sshArgs $remoteUnix
  if ($LASTEXITCODE -ne 0) { Err 'Service restart failed'; exit $LASTEXITCODE }
}
elseif ($Run.IsPresent) {
  Info 'Starting binary in background with nohup ...'
  $remote = @'
set -e
cd $REMOTE_DIR/firmware
BIN=plants_love_rust_firmware
pgrep -fa "$BIN" >/dev/null 2>&1 && pkill -f "$BIN" || true
nohup ./target/release/$BIN > run.log 2>&1 &
sleep 0.5
if ! pgrep -fa "$BIN" >/dev/null 2>&1; then
  echo "Process not running (it may have exited). Showing last 50 log lines:"
  tail -n 50 run.log || true
fi
echo "Logs: tail -f $REMOTE_DIR/firmware/run.log"
'@
  $remote = $remote.Replace('$REMOTE_DIR', $RemoteDir)
  # Normalize line endings to LF for bash on the Pi
  $remoteUnix = $remote -replace "`r", ""
  $remoteOut = & ssh @sshArgs $remoteUnix 2>&1
  $remoteCode = $LASTEXITCODE
  if ($remoteOut) { Write-Host $remoteOut }
  if ($remoteCode -ne 0) {
    Warn "Remote run returned exit code $remoteCode. The process may have exited immediately; see log snippet above."
  }
}
else {
  Info 'Upload complete. To run on the Pi:'
  Write-Host "  ssh -i `"$KeyPath`" $PiUser@$PiHost 'cd $RemoteDir/firmware && ./target/release/plants_love_rust_firmware'"
}

Write-Host "`nDeploy complete" -ForegroundColor Green