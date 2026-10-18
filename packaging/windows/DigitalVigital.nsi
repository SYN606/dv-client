; Digital Vigital (DV) — Modern Windows Installer Script (NSIS)
; Allows user installation on ANY drive or custom folder directory.

Unicode True
SetCompressor /SOLID lzma

!define PRODUCT_NAME "Digital Vigital"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "Digital Vigital"
!define PRODUCT_WEB_SITE "https://digitalvigital.com"
!define PRODUCT_EXE "dv-launcher.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"

!include "MUI2.nsh"

; MUI Configuration
!define MUI_ABORTWARNING
!define MUI_ICON "..\..\assets\logo.png"
!define MUI_UNICON "..\..\assets\logo.png"

; Default Installation Directory (user can change drive and folder freely)
InstallDir "$PROGRAMFILES64\Digital Vigital"
InstallDirRegKey HKLM "${PRODUCT_UNINST_KEY}" "InstallLocation"
RequestExecutionLevel admin

; Installer Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\README.md"
; Directory selection page: User can choose ANY drive (C:, D:, E:, etc.) and custom path
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

; Finish Page: Option to launch Digital Vigital immediately
!define MUI_FINISHPAGE_RUN "$INSTDIR\${PRODUCT_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Digital Vigital Launcher"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "MainSection" SEC01
    SetOutPath "$INSTDIR"
    SetOverwrite on

    ; Application binary and assets
    File "..\..\target\release\${PRODUCT_EXE}"
    File "..\..\assets\logo.png"
    File "..\..\README.md"

    ; Create Shortcuts in Start Menu and Desktop
    CreateDirectory "$SMPROGRAMS\Digital Vigital"
    CreateShortcut "$SMPROGRAMS\Digital Vigital\Digital Vigital.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\logo.png" 0
    CreateShortcut "$SMPROGRAMS\Digital Vigital\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    CreateShortcut "$DESKTOP\Digital Vigital.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\logo.png" 0

    ; Write Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Windows Add/Remove Programs Registry Entries
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\${PRODUCT_EXE},0"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "InstallLocation" "$INSTDIR"
    WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoModify" 1
    WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
    ; Remove application files
    Delete "$INSTDIR\${PRODUCT_EXE}"
    Delete "$INSTDIR\logo.png"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\Uninstall.exe"

    ; Remove Shortcuts
    Delete "$SMPROGRAMS\Digital Vigital\Digital Vigital.lnk"
    Delete "$SMPROGRAMS\Digital Vigital\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Digital Vigital"
    Delete "$DESKTOP\Digital Vigital.lnk"

    ; Remove Install directory if empty
    RMDir "$INSTDIR"

    ; Clean up registry
    DeleteRegKey HKLM "${PRODUCT_UNINST_KEY}"
SectionEnd
