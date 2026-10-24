; ==============================================================================
; Digital Vigital Launcher - Premium Windows Installer Script (NSIS)
; High-performance, privacy-first desktop Minecraft launcher
; Supports installation on ANY drive (C:, D:, E:, etc.) with full branding & UX
; ==============================================================================

Unicode True
SetCompressor /SOLID lzma

!define PRODUCT_NAME "Digital Vigital Launcher"
!define PRODUCT_SHORT_NAME "Digital Vigital"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "Digital Vigital"
!define PRODUCT_WEB_SITE "https://digitalvigital.fun/minecraft"
!define PRODUCT_EXE "dv-launcher-app.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"

OutFile "..\..\dist\DigitalVigital-Setup-1.0.0.exe"
BrandingText "Digital Vigital Launcher - High-Performance Minecraft Client"

; Version Information for Windows Explorer & Properties Dialog
VIProductVersion "1.0.0.0"
VIAddVersionKey "ProductName" "Digital Vigital Launcher Setup"
VIAddVersionKey "Comments" "High-performance, privacy-first Minecraft launcher"
VIAddVersionKey "CompanyName" "Digital Vigital"
VIAddVersionKey "LegalCopyright" "Copyright (c) 2026 Digital Vigital"
VIAddVersionKey "FileDescription" "Digital Vigital Launcher Setup"
VIAddVersionKey "FileVersion" "1.0.0.0"
VIAddVersionKey "ProductVersion" "1.0.0"

; Use Highest available execution level (allows seamless install for standard & admin users)
RequestExecutionLevel admin

; Default Installation Directory (allows user to install on ANY drive or custom folder)
InstallDir "$PROGRAMFILES64\Digital Vigital Launcher"
InstallDirRegKey HKLM "${PRODUCT_UNINST_KEY}" "InstallLocation"

; ==============================================================================
; MUI2 Modern User Interface Configuration
; ==============================================================================
!include "MUI2.nsh"

!define MUI_ABORTWARNING
!define MUI_ICON "..\..\assets\logo.ico"
!define MUI_UNICON "..\..\assets\logo.ico"

; Custom branded sidebar (164x314) and header (150x57) graphics
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "header.bmp"
!define MUI_HEADERIMAGE_RIGHT
!define MUI_WELCOMEFINISHPAGE_BITMAP "wizard.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "wizard.bmp"

; --- Welcome Page ---
!define MUI_WELCOMEPAGE_TITLE "Welcome to Digital Vigital Launcher Setup"
!define MUI_WELCOMEPAGE_TEXT "Digital Vigital is a high-performance, privacy-first Minecraft launcher powered by an independent Rust engine and modern Tauri v2 + Svelte desktop UI.$\r$\n$\r$\nKey Advantages:$\r$\n- Maximum Performance: High-speed async Rust engine with responsive modern UI$\r$\n- Smart GPU: Automatically routes to high-performance NVIDIA / AMD GPU$\r$\n- Mod Marketplace: 1-Click mod installs & compatibility filtering via Modrinth$\r$\n- Hardware Vault: Microsoft OAuth tokens secured in Windows Credential Manager$\r$\n$\r$\nClick Next to continue."
!insertmacro MUI_PAGE_WELCOME

; --- License Page (Rendered via clean RTF) ---
!define MUI_LICENSEPAGE_CHECKBOX
!insertmacro MUI_PAGE_LICENSE "license.rtf"

; --- Directory Selection Page ---
!define MUI_DIRECTORYPAGE_TEXT_TOP "Setup will install Digital Vigital Launcher in the following folder. To install on another drive (e.g. D:\, E:\) or custom path, click Browse and select the directory."
!insertmacro MUI_PAGE_DIRECTORY

; --- Installation Progress Page ---
!insertmacro MUI_PAGE_INSTFILES

; --- Finish Page ---
!define MUI_FINISHPAGE_TITLE "Digital Vigital Launcher is Ready!"
!define MUI_FINISHPAGE_TEXT "Digital Vigital Launcher v${PRODUCT_VERSION} has been successfully installed on your computer.$\r$\n$\r$\nYour Minecraft. Your Control."
!define MUI_FINISHPAGE_RUN "$INSTDIR\${PRODUCT_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Digital Vigital Launcher now"
!insertmacro MUI_PAGE_FINISH

; --- Uninstaller Pages ---
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

; ==============================================================================
; Initialization & Single Instance Guard
; ==============================================================================
Function .onInit
    ; Single installer instance mutex
    System::Call 'kernel32::CreateMutex(p 0, i 0, t "DigitalVigitalSetupMutex") ?e'
    Pop $R0
    StrCmp $R0 0 +3
        MessageBox MB_OK|MB_ICONEXCLAMATION "The Digital Vigital installer is already running."
        Abort
FunctionEnd

; ==============================================================================
; Main Installation Section
; ==============================================================================
Section "MainSection" SEC01
    SetOutPath "$INSTDIR"
    SetOverwrite on

    ; Application binary, assets & documentation
    File "/oname=${PRODUCT_EXE}" "..\..\dv-launcher-app.exe"
    File "..\..\WebView2Loader.dll"
    File "..\..\assets\logo.png"
    File "..\..\assets\logo.ico"
    File "..\..\README.md"
    File "..\..\LICENSE"

    ; Create Shortcuts in Start Menu and Desktop
    CreateDirectory "$SMPROGRAMS\Digital Vigital Launcher"
    CreateShortcut "$SMPROGRAMS\Digital Vigital Launcher\Digital Vigital Launcher.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\logo.ico" 0
    CreateShortcut "$SMPROGRAMS\Digital Vigital Launcher\Uninstall Digital Vigital.lnk" "$INSTDIR\Uninstall.exe" "" "$INSTDIR\logo.ico" 0
    CreateShortcut "$DESKTOP\Digital Vigital Launcher.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\logo.ico" 0

    ; Write Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Windows Add/Remove Programs Registry Entries
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\logo.ico"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "HelpLink" "https://discord.gg/flex-dv"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "Comments" "Digital Vigital (DV) - Minecraft Launcher"
    WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "EstimatedSize" 28000
    WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoModify" 1
    WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoRepair" 1
SectionEnd

; ==============================================================================
; Uninstallation Section
; ==============================================================================
Section "Uninstall"
    ; Remove application files
    Delete "$INSTDIR\${PRODUCT_EXE}"
    Delete "$INSTDIR\WebView2Loader.dll"
    Delete "$INSTDIR\logo.png"
    Delete "$INSTDIR\logo.ico"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\Uninstall.exe"

    ; Remove Shortcuts
    Delete "$SMPROGRAMS\Digital Vigital Launcher\Digital Vigital Launcher.lnk"
    Delete "$SMPROGRAMS\Digital Vigital Launcher\Uninstall Digital Vigital.lnk"
    RMDir "$SMPROGRAMS\Digital Vigital Launcher"
    ; Also cleanup legacy shortcuts if present
    Delete "$SMPROGRAMS\Digital Vigital\Digital Vigital.lnk"
    Delete "$SMPROGRAMS\Digital Vigital\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Digital Vigital"
    Delete "$DESKTOP\Digital Vigital Launcher.lnk"
    Delete "$DESKTOP\Digital Vigital.lnk"

    ; Remove Install directory if empty
    RMDir "$INSTDIR"

    ; Clean up registry
    DeleteRegKey HKLM "${PRODUCT_UNINST_KEY}"
    DeleteRegKey /ifempty HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Digital Vigital"
SectionEnd
