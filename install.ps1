# OmaWrite-RS Windows Application Installer
# Deploys OmaWrite into user programs and registers it in the Windows Start Menu and Desktop.

$ErrorActionPreference = "Stop"

$installDir = "$env:LOCALAPPDATA\Programs\OmaWrite"
$dataDir = Join-Path $installDir "omawrite_data"
$recoveryDir = Join-Path $dataDir "recovery"

Write-Host ">>> Installing OmaWrite-RS to: $installDir" -ForegroundColor Cyan

# Create directories
New-Item -ItemType Directory -Force $installDir | Out-Null
New-Item -ItemType Directory -Force $recoveryDir | Out-Null

# 1. Copy Release Binary
$releaseExe = Join-Path $PSScriptRoot "target\release\omawrite-rs.exe"
if (-not (Test-Path $releaseExe)) {
    Write-Host "Building release binary..." -ForegroundColor Yellow
    $env:Path = "C:\Users\arion\AppData\Local\Microsoft\WinGet\Packages\MartinStorsjo.LLVM-MinGW.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\llvm-mingw-20261020-ucrt-x86_64\bin;" + $env:Path
    cargo build --release
}

$destExe = Join-Path $installDir "omawrite-rs.exe"
Copy-Item -Force $releaseExe $destExe
$exeSize = (Get-Item $destExe).Length
Write-Host "[OK] Installed executable: $destExe ($exeSize bytes)" -ForegroundColor Green

# 2. Copy Icon
$icoSource = Join-Path $PSScriptRoot "omawrite_icon.ico"
$destIco = Join-Path $installDir "omawrite.ico"
if (Test-Path $icoSource) {
    Copy-Item -Force $icoSource $destIco
    Write-Host "[OK] Installed application icon: $destIco" -ForegroundColor Green
}

# 3. Create Default config.toml if missing
$configPath = Join-Path $dataDir "config.toml"
if (-not (Test-Path $configPath)) {
    $cfg = @(
        'window_width = 1150.0',
        'window_height = 780.0',
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
        'canvas_max_width = 72.0'
    )
    $cfg -join "`r`n" | Set-Content -Path $configPath -Encoding utf8
    Write-Host "[OK] Initialized default config.toml" -ForegroundColor Green
}

# 4. Create Default snippets_de.toml if missing
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
        '[[categories.items]]',
        'label = "Stellungnahme"',
        'text = "Im Folgenden moechte ich zu den wesentlichen Aspekten dieser Debatte Stellung nehmen. "',
        '',
        '[[categories.items]]',
        'label = "Haeufig diskutiert"',
        'text = "Es wird heutzutage haeufig darueber diskutiert, inwieweit diese Entwicklung positiv zu bewerten ist. "',
        '',
        '[[categories]]',
        'name = "Meinung (Opinion)"',
        '',
        '[[categories.items]]',
        'label = "Meiner Ansicht nach"',
        'text = "Meiner Ansicht nach sollte man diesen Punkt keineswegs ausser Acht lassen. "',
        '',
        '[[categories.items]]',
        'label = "Feste Ueberzeugung"',
        'text = "Ich bin der festen Ueberzeugung, dass ein Umdenken in diesem Bereich dringend erforderlich ist. "',
        '',
        '[[categories.items]]',
        'label = "Eigene Erfahrung"',
        'text = "Aus meiner persoenlichen Erfahrung laesst sich bestaetigen, dass dieser Ansatz sinnvoll ist. "',
        '',
        '[[categories.items]]',
        'label = "Ausser Zweifel"',
        'text = "Es steht voellig ausser Zweifel, dass dieser Aspekt eine zentrale Rolle einnimmt. "',
        '',
        '[[categories.items]]',
        'label = "Standpunkt"',
        'text = "Ich vertrete hierbei den Standpunkt, dass Qualitaet stets Vorrang haben sollte. "',
        '',
        '[[categories]]',
        'name = "Vor- & Nachteile (Pros/Cons)"',
        '',
        '[[categories.items]]',
        'label = "Einerseits / Andererseits"',
        'text = "Einerseits spricht dafuer, dass ... Andererseits muss bedacht werden, dass ... "',
        '',
        '[[categories.items]]',
        'label = "Entscheidender Vorteil"',
        'text = "Ein entscheidender Vorteil besteht darin, dass dadurch wertvolle Zeit gespart wird. "',
        '',
        '[[categories.items]]',
        'label = "Einwand"',
        'text = "Demgegenueber laesst sich jedoch einwenden, dass auch Risiken vorhanden sind. "',
        '',
        '[[categories.items]]',
        'label = "Nicht vernachlaessigen"',
        'text = "Nicht zu vernachlaessigen ist ferner der Gesichtspunkt der Nachhaltigkeit. "',
        '',
        '[[categories.items]]',
        'label = "Zwar ... aber dennoch"',
        'text = "Zwar laesst sich argumentieren, dass die Kosten hoch sind, dennoch ueberwiegen die Vorzuege. "',
        '',
        '[[categories]]',
        'name = "Schluss (Conclusion)"',
        '',
        '[[categories.items]]',
        'label = "Zusammenfassend"',
        'text = "Zusammenfassend laesst sich sagen, dass eine differenzierte Betrachtung notwendig ist. "',
        '',
        '[[categories.items]]',
        'label = "Schlussfolgerung"',
        'text = "Abschliessend bin ich zu dem Schluss gekommen, dass die positiven Aspekte deutlich ueberwiegen. "',
        '',
        '[[categories.items]]',
        'label = "Sinnvolle Loesung"',
        'text = "Vor diesem Hintergrund halte ich es fuer sinnvoll, einen Mittelweg einzuschlagen. "',
        '',
        '[[categories.items]]',
        'label = "Alles in allem"',
        'text = "Alles in allem zeigt sich deutlich, dass eine zukunftsorientierte Handlungsweise gefordert ist. "'
    )
    $snip -join "`r`n" | Set-Content -Path $snippetsPath -Encoding utf8
    Write-Host "[OK] Initialized default snippets_de.toml" -ForegroundColor Green
}

# 5. Create Windows Start Menu Shortcut (registers into "All apps" & Windows Search)
$wsh = New-Object -ComObject WScript.Shell
$startMenuDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$startMenuShortcut = Join-Path $startMenuDir "OmaWrite.lnk"
$shortcut = $wsh.CreateShortcut($startMenuShortcut)
$shortcut.TargetPath = $destExe
$shortcut.WorkingDirectory = $installDir
$shortcut.IconLocation = "$destIco,0"
$shortcut.Description = "OmaWrite-RS: Portable Minimalist Writing Studio"
$shortcut.Save()
Write-Host "[OK] Added to Windows Start Menu / Application List: $startMenuShortcut" -ForegroundColor Green

# 6. Create Desktop Shortcut
$desktopDir = [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::Desktop)
$desktopShortcut = Join-Path $desktopDir "OmaWrite.lnk"
$shortcutDesktop = $wsh.CreateShortcut($desktopShortcut)
$shortcutDesktop.TargetPath = $destExe
$shortcutDesktop.WorkingDirectory = $installDir
$shortcutDesktop.IconLocation = "$destIco,0"
$shortcutDesktop.Description = "OmaWrite-RS: Portable Minimalist Writing Studio"
$shortcutDesktop.Save()
Write-Host "[OK] Added Desktop Shortcut: $desktopShortcut" -ForegroundColor Green

Write-Host ""
Write-Host ">>> OmaWrite-RS installation complete! You can now launch it from:" -ForegroundColor Cyan
Write-Host "  1. Windows Start Menu -> All Apps -> 'OmaWrite'" -ForegroundColor White
Write-Host "  2. Windows Search (press Win key, type 'OmaWrite')" -ForegroundColor White
Write-Host "  3. Desktop shortcut ('OmaWrite')" -ForegroundColor White
