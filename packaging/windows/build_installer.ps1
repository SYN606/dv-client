# PowerShell Script to Build Windows Installer with Custom Drive Selection
# Requires NSIS (makensis.exe) or Inno Setup (iscc.exe)

$ErrorActionPreference = "Stop"

Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "Building Digital Vigital (DV) Windows Installer" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan

$ProjectRoot = Resolve-Path "$PSScriptRoot\..\.."
Set-Location $ProjectRoot

# 1. Compile release binary for Windows if needed
if (-not (Test-Path "$ProjectRoot\target\release\dv-launcher.exe")) {
    Write-Host "[1/3] Compiling optimized release binary..." -ForegroundColor Yellow
    cargo build --release
}

# 2. Check for NSIS compiler
$MakensisPath = Get-Command "makensis.exe" -ErrorAction SilentlyContinue

New-Item -ItemType Directory -Force -Path "$ProjectRoot\dist" | Out-Null

if ($MakensisPath) {
    Write-Host "[2/3] Compiling NSIS installer with multi-drive selector..." -ForegroundColor Green
    & makensis.exe "$ProjectRoot\packaging\windows\DigitalVigital.nsi"
    Write-Host "[3/3] Windows Installer successfully created at dist\DigitalVigital-Setup-1.0.0.exe!" -ForegroundColor Green
    exit 0
}

# 3. Check for Inno Setup compiler
$InnoPath = Get-Command "iscc.exe" -ErrorAction SilentlyContinue
if (-not $InnoPath -and (Test-Path "C:\Program Files (x86)\Inno Setup 6\iscc.exe")) {
    $InnoPath = "C:\Program Files (x86)\Inno Setup 6\iscc.exe"
}

if ($InnoPath) {
    Write-Host "[2/3] Compiling Inno Setup installer with multi-drive selector..." -ForegroundColor Green
    & $InnoPath "$ProjectRoot\packaging\windows\DigitalVigital.iss"
    Write-Host "[3/3] Windows Installer successfully created at dist\DigitalVigital-Setup-1.0.0.exe!" -ForegroundColor Green
    exit 0
}

Write-Host "NOTE: To compile the installer on Windows, install NSIS (https://nsis.sourceforge.io) or Inno Setup (https://jrsoftware.org/isdl.php)." -ForegroundColor Yellow
Write-Host "You can also run this automatically using GitHub Actions (.github/workflows/release.yml)." -ForegroundColor Yellow
