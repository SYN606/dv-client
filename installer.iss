; ==============================================================================
; Digital Vigital (DV) Launcher - Official Inno Setup Installer Script
; High-performance, modular Windows installer with split DLL architecture
; ==============================================================================

#define MyAppName "Digital Vigital Launcher"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Digital Vigital"
#define MyAppURL "https://digitalvigital.com"
#define MyAppExeName "DigitalVigitalLauncher.exe"

[Setup]
AppId={{D37E6B20-5621-4E83-9FB2-B3C57A8E3042}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL=https://github.com/SYN606/dv-client/issues
AppUpdatesURL=https://github.com/SYN606/dv-client/releases
DefaultDirName={autopf}\Digital Vigital
DefaultGroupName=Digital Vigital
AllowNoIcons=yes
LicenseFile=LICENSE
OutputDir=dist
OutputBaseFilename=DigitalVigital-Setup-{#MyAppVersion}
SetupIconFile=assets\logo.ico
UninstallDisplayIcon={app}\assets\logo.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
CloseApplications=force
RestartApplications=no
DisableWelcomePage=no
DisableDirPage=no
DisableProgramGroupPage=no
ShowLanguageDialog=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Modular Application Files: Small EXE + Core Engine DLL + WebView2 Runtime DLL
Source: "DigitalVigitalLauncher.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "dv_core.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "WebView2Loader.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "dist\DigitalVigital.cer"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\Trust-DigitalVigital.bat"; DestDir: "{app}"; Flags: ignoreversion 
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\logo.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\logo.ico"; Tasks: desktopicon

[Registry]
; Register custom URI protocol handler (dv://)
Root: HKA; Subkey: "Software\Classes\dv"; ValueType: string; ValueData: "URL:Digital Vigital Protocol"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\dv"; ValueName: "URL Protocol"; ValueType: string; ValueData: ""
Root: HKA; Subkey: "Software\Classes\dv\DefaultIcon"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"",0"
Root: HKA; Subkey: "Software\Classes\dv\shell\open\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Code]
// Helper function to verify previous processes are terminated before updating
function InitializeSetup(): Boolean;
begin
  Result := True;
end;
