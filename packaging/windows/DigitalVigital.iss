; Digital Vigital (DV) - Inno Setup Script
; Allows flexible installation to ANY custom drive or partition.

[Setup]
AppId={{8B847688-662E-4D4B-A795-B33AC6EC8B99}
AppName=Digital Vigital Launcher
AppVersion=1.2.0
AppPublisher=Digital Vigital
AppPublisherURL=https://digitalvigital.fun
AppSupportURL=https://discord.gg/flex-dv
AppUpdatesURL=https://digitalvigital.fun
; Default directory allows user to choose ANY drive (C:, D:, E:, etc.)
DefaultDirName={autopf}\Digital Vigital Launcher
DisableDirPage=no
AllowNoIcons=yes
AlwaysShowDirOnEditPage=yes
OutputDir=..\..\dist
OutputBaseFilename=DigitalVigital-Setup-1.2.0
SetupIconFile=..\..\assets\logo.ico
LicenseFile=license.rtf
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\release\dv-launcher.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\logo.png"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\logo.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Digital Vigital Launcher\Digital Vigital Launcher"; Filename: "{app}\dv-launcher.exe"; IconFilename: "{app}\logo.ico"
Name: "{autodesktop}\Digital Vigital Launcher"; Filename: "{app}\dv-launcher.exe"; IconFilename: "{app}\logo.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\dv-launcher.exe"; Description: "{cm:LaunchProgram,Digital Vigital Launcher}"; Flags: nowait postinstall skipifsilent
