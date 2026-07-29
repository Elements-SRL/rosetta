<#
.SYNOPSIS
    Builds the single-file Windows installer (rosetta_setup.exe).

.DESCRIPTION
    Automates the release steps:
      1. cargo build --release  -> builds target\release\rosetta.exe
      2. ISCC setup.iss         -> compiles the Inno Setup installer
      3. publish                -> copies the installer to the network share

    The version is read from Cargo.toml automatically and passed to the script
    as /DMyAppVersion, so the installer always reports the crate version.

.PARAMETER IsccPath
    Full path to ISCC.exe. If omitted, the script uses the ISCC_PATH
    environment variable, then ISCC.exe on PATH, then the default Inno Setup
    install locations.

.PARAMETER Version
    Override the installer version. Defaults to the version in Cargo.toml.

.PARAMETER PublishRoot
    Network folder the installer is published to. A rosetta_X.Y.Z subfolder is
    created inside it. Pass -SkipPublish to build without publishing.

.PARAMETER SkipPublish
    Build the installer but don't copy it to the network share.

.EXAMPLE
    .\build-installer.ps1

.EXAMPLE
    .\build-installer.ps1 -IsccPath "C:\Program Files\Inno Setup 7\ISCC.exe"

.EXAMPLE
    $env:ISCC_PATH = "C:\Program Files\Inno Setup 7\ISCC.exe"; .\build-installer.ps1
#>
[CmdletBinding()]
param(
    [string]$IsccPath = $env:ISCC_PATH,
    [string]$Version,
    [string]$PublishRoot = '\\192.168.0.100\Software\rosetta',
    [switch]$SkipPublish
)

$ErrorActionPreference = 'Stop'

# Run everything from the repo root (this script's own folder) so the relative
# paths inside setup.iss (target\release\..., vendor\dlls\...) resolve correctly.
$RepoRoot = $PSScriptRoot
Push-Location $RepoRoot

# Fail the script if a native .exe returns a non-zero exit code.
function Assert-LastExit([string]$What) {
    if ($LASTEXITCODE -ne 0) {
        throw "$What failed (exit code $LASTEXITCODE)."
    }
}

try {
    # --- Resolve ISCC.exe -----------------------------------------------------
    if (-not $IsccPath) {
        $onPath = Get-Command ISCC.exe -ErrorAction SilentlyContinue
        if ($onPath) {
            $IsccPath = $onPath.Source
        } else {
            $candidates = @(
                'C:\Program Files\Inno Setup 7\ISCC.exe'
                'C:\Program Files (x86)\Inno Setup 7\ISCC.exe'
                'C:\Program Files\Inno Setup 6\ISCC.exe'
                'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
            )
            $IsccPath = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
        }
    }
    if (-not $IsccPath) {
        throw "Inno Setup not found. Pass -IsccPath <ISCC.exe>, set `$env:ISCC_PATH, " +
              "or put ISCC.exe on PATH."
    }
    if (-not (Test-Path $IsccPath)) {
        throw "Missing '$IsccPath'. Check the -IsccPath value."
    }
    Write-Host "Using Inno Setup compiler: $IsccPath"

    # --- Resolve the version --------------------------------------------------
    if (-not $Version) {
        $Version = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages `
            | Where-Object { $_.name -eq 'rosetta' } | Select-Object -ExpandProperty version
        Assert-LastExit 'cargo metadata'
    }
    if (-not $Version) {
        throw "Could not determine the version from Cargo.toml."
    }
    Write-Host "Building rosetta_setup.exe for version $Version" -ForegroundColor Cyan

    # --- 1. Build rosetta.exe (release) ---------------------------------------
    Write-Host "`n[1/3] cargo build --release..." -ForegroundColor Green
    cargo build --release
    Assert-LastExit 'cargo build'

    $exe = 'target\release\rosetta.exe'
    if (-not (Test-Path $exe)) {
        throw "Expected '$exe' was not produced by cargo build."
    }

    # --- 2. Compile the installer ---------------------------------------------
    # The quotes around the version survive into ISPP so it is always treated as
    # a string, never parsed as a number.
    Write-Host "`n[2/3] ISCC (compile installer)..." -ForegroundColor Green
    & $IsccPath "/DMyAppVersion=`"$Version`"" setup.iss
    Assert-LastExit 'ISCC'

    # OutputDir / OutputBaseFilename in setup.iss decide this path.
    $setup = Join-Path $RepoRoot 'target\rosetta_setup.exe'
    if (-not (Test-Path $setup)) {
        throw "ISCC reported success but '$setup' is missing."
    }
    Write-Host "Installer: $setup"

    # --- 3. Publish to the network share --------------------------------------
    if ($SkipPublish) {
        Write-Host "`n[3/3] publish skipped (-SkipPublish)." -ForegroundColor Yellow
    } else {
        Write-Host "`n[3/3] publish to $PublishRoot..." -ForegroundColor Green
        if (-not (Test-Path $PublishRoot)) {
            throw "Cannot reach '$PublishRoot'. Check the share is mounted and you " +
                  "have access, or re-run with -SkipPublish. The installer is " +
                  "already built at '$setup'."
        }

        $destDir = Join-Path $PublishRoot "rosetta_$Version"
        if (Test-Path (Join-Path $destDir 'rosetta_setup.exe')) {
            Write-Warning "rosetta_$Version is already published; overwriting it."
        }
        New-Item -ItemType Directory -Force -Path $destDir | Out-Null
        Copy-Item $setup -Destination $destDir -Force

        Write-Host "`nDone. Published: $(Join-Path $destDir 'rosetta_setup.exe')" -ForegroundColor Cyan
    }
}
finally {
    Pop-Location
}
