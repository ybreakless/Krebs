# Packaging script for Krebs on Windows
param (
    [string]$Target = "x86_64-pc-windows",
    [string]$Version = "0.1.0",
    [string]$OutputDir = "dist"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Resolve-Path "$ScriptDir\..\.."
Set-Location $ProjectRoot

Write-Host ">>> Packaging Krebs v$Version for $Target..." -ForegroundColor Cyan

$ReleaseDir = "$OutputDir\krebs-v$Version-$Target"
if (Test-Path $ReleaseDir) {
    Remove-Item -Recurse -Force $ReleaseDir
}
New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null

$ExeSource = "target\release\krebs.exe"
if (-not (Test-Path $ExeSource)) {
    Write-Host ">>> Building release binary..." -ForegroundColor Yellow
    cargo build --release
}

Copy-Item $ExeSource "$ReleaseDir\Krebs.exe" -Force
Copy-Item "krebs_icon.ico" "$ReleaseDir\Krebs.ico" -Force
Copy-Item "assets\logo.svg" "$ReleaseDir\logo.svg" -Force
Copy-Item "assets\logo.png" "$ReleaseDir\logo.png" -Force
Copy-Item "install.ps1" "$ReleaseDir\install.ps1" -Force
Copy-Item "krabbe.cmd" "$ReleaseDir\krabbe.cmd" -Force
Copy-Item "krabbe" "$ReleaseDir\krabbe" -Force
Copy-Item "README.md" "$ReleaseDir\README.md" -Force
Copy-Item "LICENSE" "$ReleaseDir\LICENSE" -Force

# Copy any bundled runtime DLLs if present
Get-ChildItem -Filter "*.dll" | ForEach-Object {
    Copy-Item $_.FullName "$ReleaseDir\$($_.Name)" -Force
}

$ZipPath = "$OutputDir\krebs-v$Version-$Target.zip"
if (Test-Path $ZipPath) {
    Remove-Item -Force $ZipPath
}

Write-Host ">>> Creating zip archive: $ZipPath..." -ForegroundColor Green
Compress-Archive -Path "$ReleaseDir\*" -DestinationPath $ZipPath -Force

Write-Host ">>> Done! Created $ZipPath" -ForegroundColor Green
