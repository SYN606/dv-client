# Digital Vigital (DV) Launcher - v1.2.0 Release Notes

**Digital Vigital (DV)** v1.2.0 introduces 1-click automated Java 21 runtime installation, modern JVM argument evaluation, Quilt profile support, Microsoft OAuth keyring auto-refresh, and revamped obsidian & ruby pixel-art branding.

---

## What's New in v1.2.0

### ⚡ One-Click Automated Java 21 Installer (Windows)
- **Automatic Runtime Missing Detection**: Flags missing Java or outdated runtimes dynamically on both the Home Dashboard and Settings view.
- **Official Enterprise Distribution**: Downloads official **Eclipse Temurin OpenJDK 21 JRE** from Adoptium API (~49 MB lightweight runtime bundle) with automated fallback to **Microsoft OpenJDK 21** and GitHub release mirrors.
- **Zero Admin Privileges Required**: Extracts into `%APPDATA%\DigitalVigital\runtimes\java-21` without requiring UAC elevation, system driver access, or registry modification.
- **Safe Archive Extraction**: Prevents directory traversal with strict Zip Slip canonical boundaries.
- **Live Visual Progress**: Displays real-time download speeds, percentage, and extraction states directly in the Slint GUI.

### 🎮 Modern Minecraft 1.13+ JVM Evaluation
- **Dynamic Argument Engine**: Evaluates Mojang modern `arguments.jvm` array rules with parameter replacement (`${natives_directory}`, `${launcher_name}`, `${launcher_version}`, `${classpath}`, etc.) and legacy JVM fallback.
- **Hardware Architecture Rules**: Evaluates CPU architecture matching (`x86_64`, `arm64`, `x86`, regex) to prevent loading ARM64 native binaries on x86_64 machines.
- **macOS Native Hook**: Automatically injects `-XstartOnFirstThread` when executing on macOS.

### 🔑 Microsoft OAuth Session Auto-Refresh
- Eliminates session invalidation on restart: securely fetches and refreshes tokens via OS keyring (`CredentialVault`) before every launch attempt, guaranteeing access tokens are never transmitted as `"0"`.

### 🧩 Expanded Mod Loader Compatibility
- **Quilt Loader**: Full profile resolution, library downloading, and Quilt main-class execution.
- **OptiFine Diagnostics**: Detects missing OptiFine base JARs in the versions directory and warns users before classloader crashes occur.

### 🛡️ Revamped Obsidian & Ruby Pixel-Art Branding
- **Multi-Resolution Windows Icon (`logo.ico`)**: Includes 256×256, 128×128, 64×64, 48×48, 32×32, and 16×16 icon layers with alpha transparency.
- **PE Resource & UI Integration**: Embedded directly into the Windows executable (`dv-launcher.exe`), Slint UI header, dock/taskbar, and NSIS installer wizard panels.

---

## Downloads & Available Packages (v1.2.0)

| Package | Size | Description |
| :--- | :---: | :--- |
| [**`DigitalVigital-Setup-1.2.0.exe`**](https://github.com/SYN606/dv-client/releases/download/v1.2.0/DigitalVigital-Setup-1.2.0.exe) | **7.8 MB** | **Windows NSIS Installer**: Supports custom drive selection (`C:`, `D:`, `E:`), Start Menu & Desktop shortcuts, clean uninstaller, Windows Add/Remove Programs registration, and pixel-art branding. |
| [**`DigitalVigital-v1.2.0-windows-x64-portable.zip`**](https://github.com/SYN606/dv-client/releases/download/v1.2.0/DigitalVigital-v1.2.0-windows-x64-portable.zip) | **10.5 MB** | **Standalone Portable Bundle**: Includes `dv-launcher.exe`, high-res logos (`.png`, `.ico`), `README.md`, and `LICENSE` for plug-and-play usage without installation. |

---

## Verified SHA-256 Checksums (v1.2.0)
```text
54a60f57fa370dde68ee003c5f3c2b1a9aafc8185c02aa2662b47401762f82b3  DigitalVigital-Setup-1.2.0.exe
35555a55b408f3f61c75df0f10404365398b8fdd5987b8cf96515d73676cf72  DigitalVigital-v1.2.0-windows-x64-portable.zip
676d554e2ec70a0c291a08becf84060ad25af03d25d77f9aca4c28a4b7727c22  dv-launcher.exe
```

---

# Digital Vigital (DV) Launcher - v1.0.0 Release Notes

**Digital Vigital (DV)** is a next-generation, high-performance Minecraft launcher built from the ground up in native Rust and Slint UI. Designed around the core philosophy of **minimalist UI, extreme performance, zero telemetry, and maximum gaming efficiency**, DV delivers instantaneous startup, isolated profiles, and background RAM conservation.

## Key Highlights & Features (v1.0.0)

### Extreme Performance & Ultra-Low RAM
- **On-Launch RAM Minimization**: Automatically hides or closes the launcher window when Minecraft starts, dropping background memory consumption to near zero. Restores immediately to the foreground when the game exits.
- **Native Rust Engine**: Completely replaces bloated Electron/Chromium and Python runtimes with native machine code and Slint hardware-accelerated rendering.
- **Direct JVM Supervision**: Spawns and manages the Minecraft JVM directly without shell execution wrappers for maximum FPS stability and process isolation.

### Windows Native Architecture & Optimizations
- **Pure Windows Subsystem**: Built with `#![windows_subsystem = "windows"]` to ensure zero black Command Prompt windows open behind the launcher.
- **Flicker-Free Experience**: Applied `CREATE_NO_WINDOW` across all process executions (Java version inspection, GPU detection, browser opening) to eliminate terminal flashing.
- **Enterprise Windows JDK Detection**: Automatically discovers installed Java runtimes across all standard Windows locations.
- **Hybrid GPU Performance Routing**:
  - **NVIDIA**: Injects `SHIM_MCCOMPAT = 0x800000001` to force Windows hybrid graphics scheduling to route Minecraft to the discrete NVIDIA GPU.
  - **AMD Radeon**: Applies driver performance flags (`GPU_MAX_ALLOC_PERCENT=100`, `GPU_USE_SYNC_OBJECTS=1`, `GPU_SINGLE_ALLOC_PERCENT=100`, `GPU_MAX_HEAP_SIZE=100`).
- **Path Normalization**: Sanitizes all classpaths, library paths, and working directories with `dunce` to prevent extended-length `\\?\` UNC prefixes that break Java classloaders and LWJGL natives.

### Minimalist Modern UI / UX
- **Obsidian & Crimson Aesthetics**: Modern dark mode with translucent glass cards, crisp typography, and fluid responsive layouts.
- **Responsive Profile Grid**: Available profiles adapt smoothly to window resizing with vertically centered "ACTIVE" badges and single-click switching.

### Profile Tools & Integrity Verification
- **File Verification Pipeline**: Validates client JAR and game libraries against Mojang SHA-1 hashes and automatically re-downloads missing or corrupt files.
- **Asset Repair**: Re-verifies all asset index objects and repairs broken sound, texture, and language files with live progress reporting.
- **Modrinth Mod Studio**: Built-in online mod browser with live search, loader/version filters, and one-click mod downloads.
- **Instance Isolation**: Complete profile isolation with dedicated mod directories, config files, and custom JVM arguments.

### Privacy, Security & Account Management
- **Zero Telemetry**: No external analytics, tracking cookies, or diagnostic pings.
- **Microsoft OAuth 2.0 PKCE**: Secure authentication with encrypted credential storage.
- **Offline Account Validation**: Strictly validates offline usernames (alphanumeric, 3-16 characters, no spaces) and requires profile configuration before game launch.
- **Credential Redaction**: Automatically scrubs access tokens and bearer tokens from real-time log viewers.
