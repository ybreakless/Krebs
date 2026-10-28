# ==============================================================================
# Krebs Studio Universal Windows Installer & Terminal Pathway Setup
#
# Usage:
#   Local:  powershell -ExecutionPolicy Bypass -File .\install.ps1
#   Remote: irm https://raw.githubusercontent.com/ybreakless/Krebs/main/install.ps1 | iex
# ==============================================================================

$ErrorActionPreference = "Stop"

$installDir = "$env:LOCALAPPDATA\Programs\Krebs"
$dataDir = Join-Path $installDir "krebs_data"
$recoveryDir = Join-Path $dataDir "recovery"

Write-Host ">>> Installing Krebs Studio to: $installDir" -ForegroundColor Cyan

# Create directories
New-Item -ItemType Directory -Force $installDir | Out-Null
New-Item -ItemType Directory -Force $recoveryDir | Out-Null

# Stop running instances if any
Get-Process -Name "Krebs" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "krabbe" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 400

$baseDir = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
$releaseExe = Join-Path $baseDir "target\release\krebs.exe"
$destExe = Join-Path $installDir "Krebs.exe"

# 1. Locate, Download, or Build Release Binary
if (Test-Path $releaseExe) {
    Copy-Item -Force $releaseExe $destExe
} else {
    # Check if Krebs.exe is directly present in current folder (e.g. from release zip)
    $localExe = Join-Path $baseDir "Krebs.exe"
    if (Test-Path $localExe) {
        Copy-Item -Force $localExe $destExe
    } else {
        # Attempt to download from GitHub Releases
        $tag = "v0.1.0"
        $zipUrl = "https://github.com/ybreakless/Krebs/releases/download/$tag/krebs-$tag-x86_64-pc-windows.zip"
        $tmpZip = Join-Path $env:TEMP "krebs-release.zip"
        $tmpExtract = Join-Path $env:TEMP "krebs-extract"

        Write-Host ">>> Fetching latest release package from GitHub..." -ForegroundColor Cyan
        try {
            Invoke-WebRequest -Uri $zipUrl -OutFile $tmpZip -UseBasicParsing
            Expand-Archive -Path $tmpZip -DestinationPath $tmpExtract -Force
            $foundExe = Get-ChildItem -Path $tmpExtract -Recurse -Filter "Krebs.exe" | Select-Object -First 1
            if ($foundExe) {
                Copy-Item -Force $foundExe.FullName $destExe
                # Also copy bundled dlls if any
                Get-ChildItem -Path $foundExe.DirectoryName -Filter "*.dll" | ForEach-Object {
                    Copy-Item -Force $_.FullName (Join-Path $installDir $_.Name)
                }
            }
            Remove-Item -Force $tmpZip -ErrorAction SilentlyContinue
            Remove-Item -Recurse -Force $tmpExtract -ErrorAction SilentlyContinue
        } catch {
            Write-Host "Downloading release binary failed. Attempting cargo build..." -ForegroundColor Yellow
            if (Get-Command "cargo" -ErrorAction SilentlyContinue) {
                cargo build --release
                Copy-Item -Force $releaseExe $destExe
            } else {
                throw "Could not acquire Krebs binary. Please ensure network access or install Rust (https://rustup.rs)."
            }
        }
    }
}

$exeSize = (Get-Item $destExe).Length
Write-Host "[OK] Installed executable: $destExe ($exeSize bytes)" -ForegroundColor Green

# 2. Bundle Essential Runtime DLLs (libunwind.dll, libc++.dll, libwinpthread-1.dll)
$llvmBin = "C:\Users\arion\AppData\Local\Microsoft\WinGet\Packages\MartinStorsjo.LLVM-MinGW.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\llvm-mingw-20261020-ucrt-x86_64\bin"
$dllNames = @("libunwind.dll", "libc++.dll", "libwinpthread-1.dll")
$releaseDir = Split-Path -Parent $releaseExe

foreach ($dll in $dllNames) {
    $srcDll = Join-Path $releaseDir $dll
    if (-not (Test-Path $srcDll)) {
        $srcDll = Join-Path $baseDir $dll
    }
    if ((-not (Test-Path $srcDll)) -and (Test-Path $llvmBin)) {
        $srcDll = Join-Path $llvmBin $dll
    }
    if (Test-Path $srcDll) {
        Copy-Item -Force $srcDll (Join-Path $installDir $dll)
        Write-Host "[OK] Bundled runtime dependency: $dll" -ForegroundColor Green
    }
}

# 3. Copy Icon
$icoSource = Join-Path $baseDir "krebs_icon.ico"
$destIco = Join-Path $installDir "Krebs.ico"
if (Test-Path $icoSource) {
    Copy-Item -Force $icoSource $destIco
    Write-Host "[OK] Installed application icon: $destIco" -ForegroundColor Green
}

# 4. Terminal Pathway Setup: 'krabbe' command
$krabbeExe = Join-Path $installDir "krabbe.exe"
Copy-Item -Force $destExe $krabbeExe
Write-Host "[OK] Created terminal executable: $krabbeExe" -ForegroundColor Green

$krabbeCmd = Join-Path $installDir "krabbe.cmd"
$cmdScript = "@echo off`r`nstart `"`" `"%~dp0Krebs.exe`" %*"
[System.IO.File]::WriteAllText($krabbeCmd, $cmdScript)
Write-Host "[OK] Created terminal command wrapper: $krabbeCmd" -ForegroundColor Green

$krabbeSh = Join-Path $installDir "krabbe"
$shScript = "#!/bin/sh`nDIR=`"`$(cd `"`$(dirname `"`$0`")`" && pwd)`"`nexec `"`$DIR/Krebs.exe`" `"`$@`"`n"
[System.IO.File]::WriteAllText($krabbeSh, $shScript)
Write-Host "[OK] Created bash terminal wrapper: $krabbeSh" -ForegroundColor Green

# 5. Add $installDir to User PATH Environment Variable
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($null -eq $userPath) { $userPath = "" }
if ($userPath -notlike "*$installDir*") {
    $newPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $installDir } else { "$userPath;$installDir" }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "[OK] Added $installDir to permanent User PATH environment variable" -ForegroundColor Green
}
# Also make it immediately available in current process
if ($env:Path -notlike "*$installDir*") {
    $env:Path = "$installDir;" + $env:Path
}

# 6. Create Default config.toml if missing
$configPath = Join-Path $dataDir "config.toml"
if (-not (Test-Path $configPath)) {
    $cfg = @(
        'window_width = 1200.0',
        'window_height = 820.0',
        'zoom_level = 1.0',
        'theme = "dark"',
        'target_min_words = 200',
        'target_max_words = 250',
        'timer_duration_secs = 2700',
        'recent_files = []',
        'show_left_rail = true',
        'show_right_drawer = false',
        'show_zwnj_indicator = true',
        'direction_mode = "auto"',
        'smart_quotes_de = true',
        'smart_dashes = true',
        'canvas_max_width = 120.0',
        'canvas_width_mode = "wide"',
        'font_choice = "jetbrains_mono"',
        'view_mode = "edit"',
        'app_mode = "writing"',
        'paper_pattern = "dot"',
        'accent_color = "cyan"',
        'has_seen_tour = false',
        'show_top_toolbar = true',
        'split_ratio = 0.50',
        'drawer_width = 380.0',
        'ai_provider = "hermes"',
        'ai_endpoint = "http://localhost:11434/v1"',
        'ai_model = "hermes3"',
        'ai_api_key = ""'
    )
    $cfg -join "`r`n" | Set-Content -Path $configPath -Encoding utf8
    Write-Host "[OK] Initialized default config.toml" -ForegroundColor Green
}

# 7. Create Default snippets_de.toml if missing
$snippetsPath = Join-Path $dataDir "snippets_de.toml"
if (-not (Test-Path $snippetsPath)) {
    $snip = @(
        '[[categories]]',
        'name = "Einleitung (Intro)"',
        '',
        '[[categories.items]]',
        'label = "Wichtige Rolle"',
        'text = "In der heutigen Zeit spielt dieses Thema eine immer wichtigere Rolle. "',
        '',
        '[[categories.items]]',
        'label = "Einleitend betonen"',
        'text = "Einleitend moechte ich betonen, dass die vorliegende Thematik von grosser Relevanz ist. "',
        '',
        '[[categories]]',
        'name = "Meinung (Opinion)"',
        '',
        '[[categories.items]]',
        'label = "Meiner Ansicht nach"',
        'text = "Meiner Ansicht nach sollte man diesen Punkt keineswegs ausser Acht lassen. "',
        '',
        '[[categories]]',
        'name = "Vor- & Nachteile (Pros/Cons)"',
        '',
        '[[categories.items]]',
        'label = "Einerseits / Andererseits"',
        'text = "Einerseits spricht dafuer, dass ... Andererseits muss bedacht werden, dass ... "',
        '',
        '[[categories]]',
        'name = "Schluss (Conclusion)"',
        '',
        '[[categories.items]]',
        'label = "Zusammenfassend"',
        'text = "Zusammenfassend laesst sich sagen, dass eine differenzierte Betrachtung notwendig ist. "'
    )
    $snip -join "`r`n" | Set-Content -Path $snippetsPath -Encoding utf8
    Write-Host "[OK] Initialized default snippets_de.toml" -ForegroundColor Green
}

# 8. Create Windows Start Menu Shortcut
$wsh = New-Object -ComObject WScript.Shell
$startMenuDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$startMenuShortcut = Join-Path $startMenuDir "Krebs.lnk"
$shortcut = $wsh.CreateShortcut($startMenuShortcut)
$shortcut.TargetPath = $destExe
$shortcut.WorkingDirectory = $installDir
$shortcut.IconLocation = "$destIco,0"
$shortcut.Description = "Krebs Studio: Minimalist Writing & Whiteboard"
$shortcut.Save()
Write-Host "[OK] Added to Windows Start Menu / Application List: $startMenuShortcut" -ForegroundColor Green

# 9. Create Desktop Shortcut
$desktopDir = [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::Desktop)
$desktopShortcut = Join-Path $desktopDir "Krebs.lnk"
$shortcutDesktop = $wsh.CreateShortcut($desktopShortcut)
$shortcutDesktop.TargetPath = $destExe
$shortcutDesktop.WorkingDirectory = $installDir
$shortcutDesktop.IconLocation = "$destIco,0"
$shortcutDesktop.Description = "Krebs Studio: Minimalist Writing & Whiteboard"
$shortcutDesktop.Save()
Write-Host "[OK] Added Desktop Shortcut: $desktopShortcut" -ForegroundColor Green

Write-Host ""
Write-Host ">>> Krebs Studio installation complete!" -ForegroundColor Cyan
Write-Host "Terminal Pathway:" -ForegroundColor Yellow
Write-Host "  Launch from any terminal (PowerShell, CMD, Bash) using:" -ForegroundColor White
Write-Host "      krabbe" -ForegroundColor Green
Write-Host "  Or open a file directly:" -ForegroundColor White
Write-Host "      krabbe notes.md" -ForegroundColor Green
Write-Host ""
Write-Host "Application Shortcuts:" -ForegroundColor Yellow
Write-Host "  1. Windows Start Menu -> All Apps -> 'Krebs'" -ForegroundColor White
Write-Host "  2. Windows Search (press Win key, type 'Krebs' or 'Krabbe')" -ForegroundColor White
Write-Host "  3. Desktop Shortcut ('Krebs')" -ForegroundColor White
