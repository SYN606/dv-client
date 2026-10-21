# Digital Vigital (DV) Launcher - v1.2.1 Release Notes

**Digital Vigital (DV)** v1.2.1 introduces full **Multi-Java architecture (Java 8, Java 17, and Java 21)** with dynamic version matching for all Minecraft editions (legacy 1.8.9/1.12.2 to modern 1.21.4+), an automated 1-click Java 8 runtime installer, upgraded player profile switching with explicit active status badges, and professional manual profile management on fresh installs.


## What's New in v1.2.1

### ☕ Universal Java Support: Java 8 (Legacy) + Java 17 + Java 21 (Modern)
- **Automatic Minecraft Version to Java Version Resolution**:
  - **Minecraft ≤ 1.16.5** (1.8.9, 1.12.2, 1.16.5): Automatically matches **Java 8** (essential for Forge 1.8.9/1.12.2 to prevent JVM module crashing).
  - **Minecraft 1.17 to 1.20.4**: Automatically matches **Java 17**.
  - **Minecraft 1.20.5+** (1.20.5, 1.21.4): Automatically matches **Java 21**.
- **1-Click Java 8 (Legacy) Automated Downloader**:
  - Added dedicated **"Install Java 8"** button in Settings (Java & Memory).
  - Downloads official **Eclipse Temurin OpenJDK 8 JRE** from Adoptium with automated fallback to direct binary releases.
  - Installs cleanly into isolated `%APPDATA%\DigitalVigital\runtimes\java-8` without administrative elevation.
- **Visual Java Version Badges**:
  - Each detected runtime in the Settings list now displays a crisp major version badge: `[Java 8]`, `[Java 17]`, `[Java 21]` alongside path and vendor details.
- **Dynamic Profile Java Association**:
  - The launcher UI updates live when selecting a profile to display the exact auto-matched Java runtime (e.g. `Eclipse Temurin Java 8 (Auto for MC 1.8.9)`).

### 👤 Upgraded Player Profile Integration & Management
- **Instant 1-Click Profile Switching**:
  - Stored accounts in the Accounts tab now feature an **[ACTIVE]** badge on the selected account and an instant **[Set Active]** button on inactive accounts.
  - Header and hero cards dynamically render account type badges (`[Microsoft]` in cyan or `[Offline]` in amber).
- **Hero Card Player Showcase**:
  - Displays avatar preview, active username, account type badge, cape status, and a quick **"Switch Account"** navigation button.
  - Clean empty state with **"+ Add Player"** when no player account is configured yet.
- **Accurate Launch Passing**:
  - Player credentials, offline UUIDs, and Microsoft auth tokens are passed directly to Minecraft launch arguments with in-game skin pack sync.

### 🎯 Professional Fresh-Install Experience
- **No Auto-Created Instances**:
  - Fresh launcher installs no longer auto-create arbitrary Fabric instances.
  - Displays a clean initial state with `"No Profile Selected"`, welcoming the user to pick or create their preferred Minecraft version and mod loader.
- **Colored Loader Badges**:
  - Profile cards and shelf entries show styled loader badges: Fabric (Cyan), Forge (Amber), Quilt (Purple), NeoForge (Coral), Vanilla (Emerald).


## Downloads & Available Packages (v1.2.1)

| Package | Size | Description |
| :--- | :---: | :--- |
| [**`DigitalVigital-Setup-1.2.1.exe`**](https://github.com/SYN606/dv-client/releases/download/v1.2.1/DigitalVigital-Setup-1.2.1.exe) | **7.8 MB** | **Windows NSIS Installer**: Supports custom drive selection (`C:`, `D:`, `E:`), Start Menu & Desktop shortcuts, clean uninstaller, Windows Add/Remove Programs registration, and pixel-art branding. |
| [**`DigitalVigital-v1.2.1-windows-x64-portable.zip`**](https://github.com/SYN606/dv-client/releases/download/v1.2.1/DigitalVigital-v1.2.1-windows-x64-portable.zip) | **10.6 MB** | **Standalone Portable Bundle**: Includes `dv-launcher.exe`, high-res logos (`.png`, `.ico`), `README.md`, and `LICENSE` for plug-and-play usage without installation. |


## Verified SHA-256 Checksums (v1.2.1)
```text
fd5bff6124777bda4649ac54a5503fd7a6e10721f97f956d07dc2193cef4264a  DigitalVigital-Setup-1.2.1.exe
5fe14ec08a5c21d0cf35e9bd4ab8ceafc10c03c1d651f451d5e08b0771ccc2ad  DigitalVigital-v1.2.1-windows-x64-portable.zip
0f0eff199a805f1a4c59a70db216a3819ed2bd05e9498443493734b19192c54b  dv-launcher.exe
```


# Previous Releases

## Digital Vigital (DV) Launcher - v1.2.0 Release Notes

**Digital Vigital (DV)** v1.2.0 introduces 1-click automated Java 21 runtime installation, modern JVM argument evaluation, Quilt profile support, Microsoft OAuth keyring auto-refresh, and revamped obsidian & ruby pixel-art branding.

### What's New in v1.2.0
- **One-Click Automated Java 21 Installer (Windows)**: Adoptium OpenJDK 21 JRE download & extraction into isolated runtimes directory.
- **Modern Minecraft 1.13+ JVM Evaluation**: Complete Mojang argument engine with variable replacement and CPU architecture matching.
- **Microsoft OAuth Session Auto-Refresh**: Seamless token refresh via OS keyring vault.
- **Expanded Mod Loader Compatibility**: Quilt profile resolution and OptiFine diagnostics.
- **Obsidian & Ruby Pixel-Art Branding**: Multi-resolution Windows icon (`logo.ico`).
