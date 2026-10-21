# Digital Vigital (DV) Launcher - v1.3.0 Release Notes

**Digital Vigital (DV)** v1.3.0 introduces full **Multi-Java architecture (Java 8, Java 17, and Java 21)** with dynamic version matching for all Minecraft editions (legacy 1.8.9/1.12.2 to modern 1.21.4+), an automated 1-click Java 8 runtime installer, upgraded player profile switching with explicit active status badges, and professional manual profile management on fresh installs.

---

## What's New in v1.3.0

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

---

## Downloads & Available Packages (v1.3.0)

| Package | Size | Description |
| :--- | :---: | :--- |
| [**`DigitalVigital-Setup-1.3.0.exe`**](https://github.com/SYN606/dv-client/releases/download/v1.3.0/DigitalVigital-Setup-1.3.0.exe) | **7.8 MB** | **Windows NSIS Installer**: Supports custom drive selection (`C:`, `D:`, `E:`), Start Menu & Desktop shortcuts, clean uninstaller, Windows Add/Remove Programs registration, and pixel-art branding. |
| [**`DigitalVigital-v1.3.0-windows-x64-portable.zip`**](https://github.com/SYN606/dv-client/releases/download/v1.3.0/DigitalVigital-v1.3.0-windows-x64-portable.zip) | **10.6 MB** | **Standalone Portable Bundle**: Includes `dv-launcher.exe`, high-res logos (`.png`, `.ico`), `README.md`, and `LICENSE` for plug-and-play usage without installation. |

---

## Verified SHA-256 Checksums (v1.3.0)
```text
44bdc5df033505d4cbf714bf56a6929213f1d7fd74a2d154dfede2fa3a80188f  DigitalVigital-Setup-1.3.0.exe
ab2df739922808d8f2d5f8a561acb6d502aa052458a57f4a30731ec330a54659  DigitalVigital-v1.3.0-windows-x64-portable.zip
e48a96dcbefdddb4b4de9367da5a6a00b618833f83c806f2c09a5374a3c74a76  dv-launcher.exe
```

---

# Previous Releases

## Digital Vigital (DV) Launcher - v1.2.0 Release Notes

**Digital Vigital (DV)** v1.2.0 introduces 1-click automated Java 21 runtime installation, modern JVM argument evaluation, Quilt profile support, Microsoft OAuth keyring auto-refresh, and revamped obsidian & ruby pixel-art branding.

### What's New in v1.2.0
- **One-Click Automated Java 21 Installer (Windows)**: Adoptium OpenJDK 21 JRE download & extraction into isolated runtimes directory.
- **Modern Minecraft 1.13+ JVM Evaluation**: Complete Mojang argument engine with variable replacement and CPU architecture matching.
- **Microsoft OAuth Session Auto-Refresh**: Seamless token refresh via OS keyring vault.
- **Expanded Mod Loader Compatibility**: Quilt profile resolution and OptiFine diagnostics.
- **Obsidian & Ruby Pixel-Art Branding**: Multi-resolution Windows icon (`logo.ico`).
