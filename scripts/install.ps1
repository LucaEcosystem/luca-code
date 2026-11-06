# Luca Code installer for Windows (PowerShell 5.1+).
#
# Installs the `luca` CLI by building from source with Cargo:
#   .\scripts\install.ps1            # from inside a Luca Code checkout
#   $env:LUCA_REPO_URL = "<url>"; .\scripts\install.ps1   # clone then install
#
# Override with $env:LUCA_REPO_URL to install from a fork or mirror.
# Re-running this script upgrades an existing install.
$ErrorActionPreference = "Stop"

$LucaRepoUrl = if ($env:LUCA_REPO_URL) { $env:LUCA_REPO_URL } else { "https://github.com/LucaEcosystem/luca-code.git" }
$LucaRef = if ($env:LUCA_REF) { $env:LUCA_REF } else { "main" }

function Write-Log($Message) { Write-Host "luca installer: $Message" }

function Ensure-Rust {
    if (Get-Command cargo -ErrorAction SilentlyContinue) { return }
    Write-Log "Rust (cargo) not found; installing via rustup..."
    $init = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $init
    & $init -y --profile minimal --default-toolchain stable
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    $env:Path = "$cargoBin;$env:Path"
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "cargo is still missing after rustup. Open a new terminal and re-run."
    }
}

$sourceDir = $null
if ((Test-Path ".\Cargo.toml") -and (Select-String -Path ".\Cargo.toml" -Pattern 'name = "luca-code"' -Quiet)) {
    $sourceDir = (Get-Location).Path
    Write-Log "using local checkout at $sourceDir"
} else {
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        throw "git is required to download Luca Code. Install git, then re-run."
    }
    $workdir = Join-Path ([System.IO.Path]::GetTempPath()) ("luca-code-" + [System.Guid]::NewGuid().ToString("N"))
    try {
        Write-Log "cloning $LucaRepoUrl ($LucaRef)..."
        git clone --depth 1 --branch $LucaRef $LucaRepoUrl (Join-Path $workdir "luca-code")
        if ($LASTEXITCODE -ne 0) {
            throw "clone failed. If the repository is still private, clone it yourself and re-run from inside the checkout."
        }
        $sourceDir = Join-Path $workdir "luca-code"
    } catch {
        if (Test-Path $workdir) { Remove-Item -Recurse -Force $workdir }
        throw
    }
}

Ensure-Rust

Write-Log "building and installing luca (this takes a minute)..."
cargo install --force --path $sourceDir

if (-not (Get-Command luca -ErrorAction SilentlyContinue)) {
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    throw "'luca' installed to $cargoBin but it is not on your PATH. Add $cargoBin to your user PATH, restart the terminal, and re-run."
}

Write-Log ("installed: " + (& luca --version))
Write-Log 'try it: "print(""Hello, Luca!"")" | Out-File hello.lucc; luca hello.lucc'
