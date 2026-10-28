@echo off
rem Krebs / Krabbe Terminal Launcher
setlocal
set "SCRIPT_DIR=%~dp0"
if exist "%SCRIPT_DIR%target\release\krebs.exe" (
    start "" "%SCRIPT_DIR%target\release\krebs.exe" %*
    exit /b 0
)
if exist "%LOCALAPPDATA%\Programs\Krebs\Krebs.exe" (
    start "" "%LOCALAPPDATA%\Programs\Krebs\Krebs.exe" %*
    exit /b 0
)
echo Krebs executable not found. Running with cargo...
cargo run --release -- %*
endlocal
