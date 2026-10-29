; ---------------------------------------------------------------------------
; Krebs Studio - per-user Windows installer (Inno Setup 6)
;
; CI invocation:
;   ISCC.exe /DAppVersion=x.y.z packaging\windows\installer.iss
;
; Installs to %LOCALAPPDATA%\Programs\Krebs (per-user, no admin rights) so the
; app's portable data directory (exe_dir\krebs_data, see src/config.rs) stays
; writable for the person running the app.
; ---------------------------------------------------------------------------

#ifndef AppVersion
#define AppVersion "0.0.0"
#endif

[Setup]
AppId={{9F3D4E17-72AF-4E6B-A1D5-4D2C57FBF028}}
AppName=Krebs Studio
AppVersion={#AppVersion}
AppPublisher=ybreakless
AppPublisherURL=https://github.com/ybreakless/Krebs
AppSupportURL=https://github.com/ybreakless/Krebs/issues
DefaultDirName={userpf}\Krebs
DefaultGroupName=Krebs Studio
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=commandline
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
LicenseFile=..\..\LICENSE
OutputDir=..\..\dist
OutputBaseFilename=krebs-v{#AppVersion}-x64-setup
SetupIconFile=..\..\krebs_icon.ico
UninstallDisplayIcon={app}\Krebs.ico

[Messages]
WelcomeLabel2=This installs Krebs Studio, the minimalist writing & whiteboard studio, on your computer (no admin rights required).%n%nKrebs keeps its settings next to the program, so uninstalling later won't lose your notes.

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\x86_64-pc-windows-msvc\release\krebs.exe"; DestDir: "{app}"; DestName: "Krebs.exe"; Flags: ignoreversion
Source: "..\..\krabbe.cmd"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\krabbe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\krebs_icon.ico"; DestDir: "{app}"; DestName: "Krebs.ico"; Flags: ignoreversion
Source: "..\..\assets\logo.svg"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\logo.png"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Krebs Studio"; Filename: "{app}\Krebs.exe"
Name: "{group}\Uninstall Krebs Studio"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Krebs Studio"; Filename: "{app}\Krebs.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Krebs.exe"; Description: "{cm:LaunchProgram,Krebs Studio}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Keep user data: krebs_data\ and omawrite_data\ are intentionally preserved
; so uninstalling never deletes notes or settings.
