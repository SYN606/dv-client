# Digital Vigital (DV) Launcher — v1.0.0 Release Notes

**Digital Vigital (DV)** is a next-generation, high-performance Minecraft launcher built from the ground up in native Rust and Slint UI. Designed around the core philosophy of **minimalist UI, extreme performance, zero telemetry, and maximum gaming efficiency**, DV delivers instantaneous startup, isolated profiles, and background RAM conservation.

---

## 🚀 Key Highlights & Features

### ⚡ Extreme Performance & Ultra-Low RAM
- **On-Launch RAM Minimization**: Automatically hides or closes the launcher window when Minecraft starts, dropping background memory consumption to near zero. Restores immediately to the foreground when the game exits.
- **Native Rust Engine**: Completely replaces bloated Electron/Chromium and Python runtimes with native machine code and Slint hardware-accelerated rendering.
- **Direct JVM Supervision**: Spawns and manages the Minecraft JVM directly without shell execution wrappers for maximum FPS stability and process isolation.

### 🪟 Windows Native Architecture & Optimizations
- **Pure Windows Subsystem**: Built with `#![windows_subsystem = "windows"]` to ensure zero black Command Prompt windows open behind the launcher.
- **Flicker-Free Experience**: Applied `CREATE_NO_WINDOW` across all process executions (Java version inspection, GPU detection, browser opening) to eliminate terminal flashing.
- **Enterprise Windows JDK Detection**: Automatically discovers installed Java runtimes across all standard Windows locations:
  - Eclipse Adoptium / Temurin
  - Microsoft OpenJDK
  - Amazon Corretto
  - Oracle Java
  - Azul Zulu
  - BellSoft Liberica
  - GraalVM
  - IBM Semeru
  - Red Hat OpenJDK
  - SAP SapMachine
  - Official Minecraft Launcher runtimes (`%APPDATA%\.minecraft\runtime`)
  - User IDE JDKs (`%USERPROFILE%\.jdks` and `.gradle\jdks`)
- **Hybrid GPU Performance Routing**:
  - **NVIDIA**: Injects `SHIM_MCCOMPAT = 0x800000001` to force Windows hybrid graphics scheduling to route Minecraft to the discrete NVIDIA GPU.
  - **AMD Radeon**: Applies driver performance flags (`GPU_MAX_ALLOC_PERCENT=100`, `GPU_USE_SYNC_OBJECTS=1`, `GPU_SINGLE_ALLOC_PERCENT=100`, `GPU_MAX_HEAP_SIZE=100`).
- **Path Normalization**: Sanitizes all classpaths, library paths, and working directories with `dunce` to prevent extended-length `\\?\` UNC prefixes that break Java classloaders and LWJGL natives.

### 🎨 Minimalist Modern UI / UX
- **Obsidian & Crimson Aesthetics**: Modern dark mode with translucent glass cards, crisp typography, and fluid responsive layouts.
- **Responsive Profile Grid**: Available profiles adapt smoothly to window resizing with vertically centered "ACTIVE" badges and single-click switching.
- **Clutter-Free Interface**: Removed decorative dot indicators and icon clutter from settings tabs, while optimizing previously empty spaces with asset cache maintenance and hybrid GPU guides.

### 👤 Universal Player Skin & Cape Studio
- **In-Game Texture Injection**: Automatically packages and enables a universal resource pack so player skins and capes appear across singleplayer worlds and multiplayer servers.
- **Custom Skin Uploader**: Upload standard 64x64 PNG skins with real-time 3D character previews and dual-layer overlay rendering.
- **Dedicated Cape Adding Option**: Upload custom cape PNG textures seamlessly, with a clean "Remove Cape" option (no forced built-in presets).

### 🛠️ Profile Tools & Integrity Verification
- **File Verification Pipeline**: Validates client JAR and game libraries against Mojang SHA-1 hashes and automatically re-downloads missing or corrupt files.
- **Asset Repair**: Re-verifies all asset index objects and repairs broken sound, texture, and language files with live progress reporting.
- **Modrinth Mod Studio**: Built-in online mod browser with live search, loader/version filters, and one-click mod downloads.
- **Instance Isolation**: Complete profile isolation with dedicated mod directories, config files, and custom JVM arguments.

### 🔒 Privacy, Security & Account Management
- **Zero Telemetry**: No external analytics, tracking cookies, or diagnostic pings.
- **Microsoft OAuth 2.0 PKCE**: Secure authentication with encrypted credential storage.
- **Offline Account Validation**: Strictly validates offline usernames (alphanumeric, 3–16 characters, no spaces) and requires profile configuration before game launch.
- **Credential Redaction**: Automatically scrubs access tokens and bearer tokens from real-time log viewers.

---

## 📦 Downloads & Available Packages

### 🪟 Windows
- **[`DigitalVigital-Setup-1.0.0.exe`](https://github.com/SYN606/dv-client/releases/download/v1.0.0/DigitalVigital-Setup-1.0.0.exe)** (14.2 MB)
  - **Multi-Drive Installer**: Choose any drive (`C:\`, `D:\`, `E:\`, or custom partition) during installation.
  - Includes Desktop shortcut, Start Menu shortcut, and Windows Control Panel uninstaller.

### 🐧 Linux
- **[`DigitalVigital-1.0.0-x86_64.AppImage`](https://github.com/SYN606/dv-client/releases/download/v1.0.0/DigitalVigital-1.0.0-x86_64.AppImage)** (18.0 MB)
  - Standalone, portable executable. Runs on any Linux distribution (Ubuntu, Fedora, Arch, Debian, openSUSE).
- **[`dv-launcher_1.0.0_amd64.deb`](https://github.com/SYN606/dv-client/releases/download/v1.0.0/dv-launcher_1.0.0_amd64.deb)** (20.0 MB)
  - Native Debian package for Ubuntu, Debian, Linux Mint, and Pop!_OS (`sudo apt install ./...`).
- **[`dv-launcher-1.0.0-1.x86_64.rpm`](https://github.com/SYN606/dv-client/releases/download/v1.0.0/dv-launcher-1.0.0-1.x86_64.rpm)** (13.0 MB)
  - Native RPM package for Fedora, RHEL, CentOS, and openSUSE (`sudo dnf install ./...`).

---

## 📋 Verified SHA-256 Checksums
```text
547929a54bea728ca9095f564a3098cb594084525ffa995cf5abfe6dea34fa6d  DigitalVigital-Setup-1.0.0.exe
5fa4878eba1041e6619815c08e87cc9c8f04fc8d0311cef25268fa6c59968204  DigitalVigital-1.0.0-x86_64.AppImage
86b17165df8b7f9d4fb8101e2b53cdc375a5528ae339ac4f761070ed41426928  dv-launcher_1.0.0_amd64.deb
6c3f2c08bfd58f47794fb0cfc6133cfb9d269a4f349c80374f8d8f5e54e348fb  dv-launcher-1.0.0-1.x86_64.rpm
```
