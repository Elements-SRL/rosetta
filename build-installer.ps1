<#
.SYNOPSIS
    Builds the single-file Windows installer (RosettaSetup.exe).

.DESCRIPTION
    Automates the release steps from docs\PACKAGING.md:
      1. cargo wix       -> builds rosetta.exe (release) and the MSI
      2. candle          -> compiles the WiX bundle (chains VC++ redist + MSI)
      3. light           -> links the bundle into target\wix\RosettaSetup.exe

    The version is read from Cargo.toml automatically, so the bundle always
    matches the MSI that cargo-wix produced.

.PARAMETER WixBin
    Folder containing candle.exe / light.exe. If omitted, the script uses the
    WIX_BIN environment variable, or falls back to candle.exe found on PATH.

.PARAMETER Version
    Override the installer version. Defaults to the version in Cargo.toml.

.EXAMPLE
    .\build-installer.ps1 -WixBin "C:\Users\lross\wix-toolset\bin"

.EXAMPLE
    $env:WIX_BIN = "C:\wix-toolset\bin"; .\build-installer.ps1
#>
[CmdletBinding()]
param(
    [string]$WixBin = $env:WIX_BIN,
    [string]$Version
)

$ErrorActionPreference = 'Stop'

# Run everything from the repo root (this script's own folder) so the relative
# paths inside bundle.wxs (vendor\..., target\wix\...) resolve correctly.
$RepoRoot = $PSScriptRoot
Push-Location $RepoRoot

# Fail the script if a native .exe returns a non-zero exit code.
function Assert-LastExit([string]$What) {
    if ($LASTEXITCODE -ne 0) {
        throw "$What failed (exit code $LASTEXITCODE)."
    }
}

try {
    # --- Resolve the WiX toolset location -------------------------------------
    if (-not $WixBin) {
        $candle = Get-Command candle.exe -ErrorAction SilentlyContinue
        if ($candle) {
            $WixBin = Split-Path $candle.Source
            Write-Host "Using WiX from PATH: $WixBin"
        } else {
            throw "WiX toolset not found. Pass -WixBin <folder> or set `$env:WIX_BIN, " +
                  "or put candle.exe/light.exe on PATH. See docs\PACKAGING.md."
        }
    }

    $candleExe = Join-Path $WixBin 'candle.exe'
    $lightExe  = Join-Path $WixBin 'light.exe'
    foreach ($tool in @($candleExe, $lightExe)) {
        if (-not (Test-Path $tool)) {
            throw "Missing '$tool'. Check the -WixBin folder. See docs\PACKAGING.md."
        }
    }

    # --- Resolve the version --------------------------------------------------
    if (-not $Version) {
        $Version = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages `
            | Where-Object { $_.name -eq 'rosetta' } | Select-Object -ExpandProperty version
        Assert-LastExit 'cargo metadata'
    }
    if (-not $Version) {
        throw "Could not determine the version from Cargo.toml."
    }
    Write-Host "Building RosettaSetup.exe for version $Version" -ForegroundColor Cyan

    # --- 1. Build rosetta.exe (release) + MSI ---------------------------------
    # cargo wix runs `cargo build --release` for us.
    Write-Host "`n[1/3] cargo wix (build release + MSI)..." -ForegroundColor Green
    cargo wix -b $WixBin
    Assert-LastExit 'cargo wix'

    $msi = "target\wix\rosetta-$Version-x86_64.msi"
    if (-not (Test-Path $msi)) {
        throw "Expected MSI '$msi' was not produced by cargo wix."
    }

    # --- 2. Compile the bundle ------------------------------------------------
    Write-Host "`n[2/3] candle (compile bundle)..." -ForegroundColor Green
    & $candleExe installer\bundle.wxs `
        -ext WixBalExtension -ext WixUtilExtension `
        "-dVersion=$Version" -out target\wix\bundle\
    Assert-LastExit 'candle'

    # --- 3. Link the final setup.exe ------------------------------------------
    Write-Host "`n[3/3] light (link RosettaSetup.exe)..." -ForegroundColor Green
    & $lightExe target\wix\bundle\bundle.wixobj `
        -ext WixBalExtension -ext WixUtilExtension `
        -out target\wix\RosettaSetup.exe
    Assert-LastExit 'light'

    $setup = Join-Path $RepoRoot 'target\wix\RosettaSetup.exe'
    if (-not (Test-Path $setup)) {
        throw "light reported success but '$setup' is missing."
    }

    Write-Host "`nDone. Installer: $setup" -ForegroundColor Cyan
}
finally {
    Pop-Location
}
