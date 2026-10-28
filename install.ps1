# ==============================================================================
# Krebs Studio Universal Windows Installer & Terminal Pathway Setup
#
# Usage:
#   Local:  powershell -ExecutionPolicy Bypass -File .\install.ps1
#   Remote: irm https://raw.githubusercontent.com/ybreakless/Krebs/main/install.ps1 | iex
#
# Acquisition order:
#   1. Locally built binary        (target\release\krebs.exe)
#   2. Binary next to this script  (e.g. extracted release zip)
#   3. GitHub release archive      (krebs-$tag-$target.zip, matches release.yml)
#   4. Build from source           (main.zip -> temp -> cargo build --release)
# ==============================================================================

$ErrorActionPreference = "Stop"

$repoOwner = "ybreakless"
$repoName  = "Krebs"
$repoUrl   = "https://github.com/$repoOwner/$repoName"
$fallbackTag = "v0.1.0"

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

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

function Get-LatestReleaseTag {
    # Returns the newest published release tag, or $fallbackTag when the
    # repository has no release yet or the lookup fails (offline, rate limit).
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repoOwner/$repoName/releases/latest" -UseBasicParsing
        if ($release.tag_name) { return [string]$release.tag_name }
    } catch {
        Write-Host "Could not query latest release, falling back to $fallbackTag" -ForegroundColor DarkGray
    }
    return $fallbackTag
}

function Get-CargoHome {
    # Returns the resolved cargo executable path, or $null when absent.
    if (Get-Command "cargo" -ErrorAction SilentlyContinue) {
        return (Get-Command "cargo").Source
    }
    $cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "$env:USERPROFILE\.cargo" }
    $cargoExe = Join-Path $cargoHome "bin\cargo.exe"
    if (Test-Path $cargoExe) { return $cargoExe }
    return $null
}

function Install-InstalledBinary([string]$SourceExe) {
    Copy-Item -Force $SourceExe $destExe
    # Carry along any runtime DLLs shipped beside the binary.
    Get-ChildItem -Path (Split-Path -Parent $SourceExe) -Filter "*.dll" -ErrorAction SilentlyContinue |
        ForEach-Object { Copy-Item -Force $_.FullName (Join-Path $installDir $_.Name) }
}

function Install-FromReleaseArchive {
    param([string]$Tag, [string]$Target)

    $zipUrl = "$repoUrl/releases/download/$Tag/krebs-$Tag-$Target.zip"
    $tmpZip = Join-Path $env:TEMP "krebs-release.zip"
    $tmpExtract = Join-Path $env:TEMP "krebs-extract"

    Write-Host ">>> Fetching latest release package from GitHub..." -ForegroundColor Cyan
    Write-Host "    $zipUrl" -ForegroundColor DarkGray
    Invoke-WebRequest -Uri $zipUrl -OutFile $tmpZip -UseBasicParsing
    Expand-Archive -Path $tmpZip -DestinationPath $tmpExtract -Force
    $foundExe = Get-ChildItem -Path $tmpExtract -Recurse -Filter "Krebs.exe" | Select-Object -First 1
    if (-not $foundExe) { throw "Release archive did not contain Krebs.exe" }
    Install-InstalledBinary $foundExe.FullName
    Remove-Item -Force $tmpZip -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $tmpExtract -ErrorAction SilentlyContinue
}

function Install-FromSource {
    # Last resort: download the source snapshot, build it, ship the binary.
    $buildDir = Join-Path $env:TEMP "krebs-build"
    $srcZip = Join-Path $buildDir "krebs-main.zip"
    $zipUrl = "$repoUrl/archive/refs/heads/main.zip"

    Write-Host ">>> Building Krebs from source (this may take a few minutes)..." -ForegroundColor Yellow

    try {
        Remove-Item -Recurse -Force $buildDir -ErrorAction SilentlyContinue
        New-Item -ItemType Directory -Force $buildDir | Out-Null
        Write-Host "    Downloading source snapshot..." -ForegroundColor DarkGray
        Invoke-WebRequest -Uri $zipUrl -OutFile $srcZip -UseBasicParsing
        Expand-Archive -Path $srcZip -DestinationPath $buildDir -Force

        # Locate the extracted project root (the folder containing Cargo.toml).
        $projectDir = Get-ChildItem -Path $buildDir -Recurse -Filter "Cargo.toml" |
            Select-Object -First 1 |
            ForEach-Object { $_.DirectoryName }
        if (-not $projectDir) { throw "Downloaded source snapshot contains no Cargo.toml" }

        Write-Host "    Running: cargo build --release" -ForegroundColor DarkGray
        Push-Location $projectDir
        try {
            & cargo build --release
            if ($LASTEXITCODE -ne 0) { throw "cargo build --release exited with code $LASTEXITCODE" }
        } finally {
            Pop-Location
        }

        $builtExe = Join-Path $projectDir "target\release\krebs.exe"
        if (-not (Test-Path $builtExe)) { throw "cargo reported success but target\release\krebs.exe is missing" }
        Install-InstalledBinary $builtExe
        Write-Host "[OK] Built and installed from source." -ForegroundColor Green
    } finally {
        Remove-Item -Recurse -Force $buildDir -ErrorAction SilentlyContinue
    }
}

# ------------------------------------------------------------------------------
# 1. Locate, Download, or Build Release Binary
# ------------------------------------------------------------------------------
$pipelineStart = [System.Diagnostics.Stopwatch]::StartNew()

if (Test-Path $releaseExe) {
    Write-Host ">>> Using locally built binary: $releaseExe" -ForegroundColor Cyan
    Install-InstalledBinary $releaseExe
} else {
    # Check if Krebs.exe is directly present in current folder (e.g. from release zip)
    $localExe = Join-Path $baseDir "Krebs.exe"
    if (Test-Path $localExe) {
        Write-Host ">>> Using bundled binary: $localExe" -ForegroundColor Cyan
        Install-InstalledBinary $localExe
    } else {
        # Pick the release channel matching this machine (release.yml builds msvc zips).
        $arch = $env:PROCESSOR_ARCHITECTURE
        $target = if ($arch -eq "ARM64") { "aarch64-pc-windows-msvc" } else { "x86_64-pc-windows-msvc" }
        $tag = Get-LatestReleaseTag

        $installed = $false
        try {
            Install-FromReleaseArchive -Tag $tag -Target $target
            Write-Host "[OK] Downloaded and installed release binary ($tag, $target)." -ForegroundColor Green
            $installed = $true
        } catch {
            Write-Host "Release download failed: $($_.Exception.Message)" -ForegroundColor Yellow
        }

        if (-not $installed) {
            $cargo = Get-CargoHome
            if ($null -eq $cargo) {
                throw @"
Neither release download nor compilation is possible.
  - No release archive reachable for $tag ($target).
  - Rust/Cargo is not installed (required to build from source).
Install Rust from https://rustup.rs and re-run, or download a zip directly from:
  $repoUrl/releases
"@
            }
            Install-FromSource
        }
    }
}

if (-not (Test-Path $destExe)) {
    throw "Internal error: $destExe was not installed."
}

$exeSize = (Get-Item $destExe).Length
Write-Host "[OK] Installed executable: $destExe ($exeSize bytes)" -ForegroundColor Green

# 2. Copy Icon
$icoSource = Join-Path $baseDir "krebs_icon.ico"
$destIco = Join-Path $installDir "Krebs.ico"
if (Test-Path $icoSource) {
    Copy-Item -Force $icoSource $destIco
    Write-Host "[OK] Installed application icon: $destIco" -ForegroundColor Green
}

# 3. Terminal Pathway Setup: 'krabbe' command
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

# 4. Add $installDir to User PATH Environment Variable
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

# 5. Create Default config.toml if missing
$configPath = Join-Path $dataDir "config.toml"
if (-not (Test-Path $configPath)) {
    $cfg = @(
        'window_width = 1200.0',
        'window_height = 820.0',
        'zoom_level = 1.0',
        'theme = "dark"',
        'language = "en"',
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

# 6. Create Default snippets_de.toml if missing
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

# 7. Create Windows Start Menu Shortcut
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

# 8. Create Desktop Shortcut
$desktopDir = [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::Desktop)
$desktopShortcut = Join-Path $desktopDir "Krebs.lnk"
$shortcutDesktop = $wsh.CreateShortcut($desktopShortcut)
$shortcutDesktop.TargetPath = $destExe
$shortcutDesktop.WorkingDirectory = $installDir
$shortcutDesktop.IconLocation = "$destIco,0"
$shortcutDesktop.Description = "Krebs Studio: Minimalist Writing & Whiteboard"
$shortcutDesktop.Save()
Write-Host "[OK] Added Desktop Shortcut: $desktopShortcut" -ForegroundColor Green

$pipelineStart.Stop()

Write-Host ""
Write-Host ">>> Krebs Studio installation complete in $([int]$pipelineStart.Elapsed.TotalSeconds) seconds!" -ForegroundColor Cyan
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
