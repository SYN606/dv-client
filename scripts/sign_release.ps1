# ==============================================================================
# Digital Vigital (DV) - Release Binary Authenticode Signer
# Signs executable binaries and setup installer with Digital Vigital certificate
# ==============================================================================

$ErrorActionPreference = "Stop"

$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
$DistDir = "$ProjectRoot\dist"
$CertSubject = "CN=Digital Vigital, O=Digital Vigital, OU=Minecraft Launcher, C=US"

Write-Host ">>> Locating or creating Digital Vigital Code Signing Certificate..." -ForegroundColor Cyan
$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -like "*CN=Digital Vigital*" } | Select-Object -First 1

if (-not $cert) {
    Write-Host "Generating official self-signed Code Signing Certificate..." -ForegroundColor Yellow
    $cert = New-SelfSignedCertificate `
        -Subject $CertSubject `
        -Type CodeSigningCert `
        -KeyUsage DigitalSignature `
        -KeySpec Signature `
        -KeyLength 2048 `
        -HashAlgorithm SHA256 `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -NotAfter (Get-Date).AddYears(5)
    Write-Host "Created certificate thumbprint: $($cert.Thumbprint)" -ForegroundColor Green
} else {
    Write-Host "Found existing certificate: $($cert.Thumbprint)" -ForegroundColor Green
}

# Export public certificate for distribution & 1-click test installation
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
$CertPath = "$DistDir\DigitalVigital.cer"
Export-Certificate -Cert $cert -FilePath $CertPath -Force | Out-Null
Write-Host "Exported public certificate to $CertPath" -ForegroundColor Green

# 1-Click Trust batch script for testers
$TrustScript = @"
@echo off
:: ==============================================================================
:: Digital Vigital - 1-Click Trusted Certificate Installer
:: Run as Administrator once to completely eliminate Windows SmartScreen &
:: untrusted publisher warnings on this machine.
:: ==============================================================================
echo ========================================================
echo Installing Digital Vigital Certificate into Trusted Root...
echo ========================================================
certutil -addstore -f "Root" "%~dp0DigitalVigital.cer"
certutil -addstore -f "TrustedPublisher" "%~dp0DigitalVigital.cer"
echo.
echo [OK] Digital Vigital Certificate installed successfully!
echo Windows SmartScreen will now recognize Digital Vigital as a trusted publisher.
pause
"@

$TrustScript | Set-Content -Path "$DistDir\Trust-DigitalVigital.bat" -Encoding ASCII
Write-Host "Created $DistDir\Trust-DigitalVigital.bat" -ForegroundColor Green

# Target binaries to sign
$targets = @(
    "$ProjectRoot\DigitalVigitalLauncher.exe",
    "$ProjectRoot\dv-launcher-app.exe",
    "$DistDir\DigitalVigital-Setup-1.0.0.exe"
)

foreach ($target in $targets) {
    if (Test-Path $target) {
        Write-Host "Signing $target..." -ForegroundColor Cyan
        try {
            # Attempt with RFC 3161 timestamping server
            Set-AuthenticodeSignature -FilePath $target -Certificate $cert -TimestampServer "http://timestamp.digicert.com" -HashAlgorithm SHA256 | Out-Null
            Write-Host "Signed $target with Digicert timestamp." -ForegroundColor Green
        } catch {
            Write-Warning "Timestamp server unreachable, signing without timestamp..."
            Set-AuthenticodeSignature -FilePath $target -Certificate $cert -HashAlgorithm SHA256 | Out-Null
            Write-Host "Signed $target." -ForegroundColor Green
        }
    }
}

Write-Host "All targets signed successfully!" -ForegroundColor Green
