# Digital Vigital (DV) Launcher - v1.0.0 Release Notes

**Digital Vigital (DV) Launcher v1.0.0** is the official milestone release — a ground-up, blazing-fast desktop Minecraft client engineered with an independent **Rust** backend, **Tauri v2**, and a modern **Svelte** UI. Built for pure performance, privacy, and zero bloat (no ads, no trackers, no paywalls), v1.0.0 brings multi-drive storage management, a 3D Skins & Capes Studio, automated crash reporting, and multi-Java runtime orchestration.

---

## What's New in v1.0.0

### 🚀 Modern Desktop Architecture (Rust + Tauri v2 + Svelte)
- **Blazing Fast Performance**: Sub-second startup times, minimal memory footprint (~40MB RAM), and instant UI responsiveness compared to heavy Electron-based launchers.
- **Hardware Acceleration**: Automatic dedicated GPU detection (NVIDIA / AMD) with intelligent routing to prevent Minecraft from running on integrated graphics.
- **Seamless Window Experience**: Native Windows borderless framing, drag areas, custom controls, and responsive layout across all display scalings.

---

### 💾 Multi-Drive Game & Instances Directory Management
- **Never Fill Your C: Drive**: Complete freedom to store Minecraft versions, libraries, runtimes, assets, and modpacks on any drive (`D:`, `E:`, external SSDs, etc.).
- **Independent Instances Directory**:
  - Keep your core game assets on one disk and separate your heavy modpacks and save files onto a high-speed NVMe or secondary hard drive.
  - Native file dialog pickers for both the **Base Game Directory** and the **Instances Directory**.
  - 1-Click **"Reset Default"** and **"Open in File Explorer"** buttons for both directories in the Settings tab.
  - Hot-reloads all instances and mod profiles dynamically without restarting the application.

---

### 🎨 3D Skins & Capes Studio
- **Real-Time 3D Stage**:
  - Offline-capable 3D canvas powered by `skinview3d` with realistic studio lighting and ambient platform glow.
  - **Direct Drag & Drop**: Drag and drop any `.png` skin or cape file directly onto the 3D model for instant preview and assignment.
  - **Quick Camera Presets**: 1-click camera snaps for **Front**, **Back**, **Left Side**, **Right Side**, **360° Auto-Spin**, and **Reset View**.
  - **Dynamic Physics & Poses**: Switch between **Walk**, **Run**, **Fly**, and **Idle** animations with a **Play/Pause** toggle and variable animation speed.
  - **Arm Thickness Rigs**: Toggle between **Classic (4px)** and **Slim (3px / Alex)** models with live texture re-mapping.
  - **Cape Visibility**: On-stage toggle to preview your character with or without equipped capes.
- **Exclusive Capes Collection**:
  - Equippable custom cosmetics with rarity badges:
    - **DV Crimson & Gold** (Official Digital Vigital Velvet & Gold Insignia)
    - **Founder Obsidian Gold** (VIP Founder Star)
    - **Ender Void Eye** (The End Dimension)
    - **Cyber Neon Circuit** (High-Tech Hex)
- **Player Skin Lookup & Stealer**:
  - Enter any Minecraft Java username to fetch their live skin texture directly from Mojang/Minotar.
  - Includes 1-click shortcuts for popular creators (*Notch*, *Technoblade*, *MumboJumbo*, *Grian*, *DanTDM*, *CaptainSparklez*).
- **Texture Export**: Download your active skin texture PNG with 1 click.

---

### ☕ Multi-Java Runtime Suite (Java 8, 17, 21)
- **Interactive Java Selector in Settings**:
  - Full visual list of all detected Java installations on the machine with version badges (**Java 8**, **Java 17**, **Java 21**).
  - 1-Click activation of any specific Java installation or "Auto-Detect (Recommended)".
  - "Browse Custom Java..." native dialog to pick any `java.exe` on any drive.
  - "Re-scan System" button to refresh installed JVMs on the fly.
  - Active Java status displayed dynamically across the sidebar and Play tab.
- **Automatic Version Resolution**:
  - Minecraft ≤ 1.16.5: Automatically routes to **Java 8** (essential for Forge 1.8.9 / 1.12.2).
  - Minecraft 1.17 – 1.20.4: Automatically routes to **Java 17**.
  - Minecraft 1.20.5+: Automatically routes to **Java 21**.
- **Automated 1-Click Java Installers**: Download and install Temurin OpenJDK runtimes directly into isolated launcher directories without requiring administrative rights.

---

### 🛡️ Automated Crash Reporting & Live Console
- **Direct Discord Webhook Integration**:
  - When Minecraft crashes or encounters game startup failure, players can submit a 1-click report directly to the development Discord.
  - Automatically packages execution logs, crash snippets, memory specs, GPU vendor, OS version, and active mod loader.
- **Live Real-Time Console**:
  - High-throughput ANSI log viewer with log export, automatic scrolling, and quick-filter diagnostics.

---

### 🔔 Update Notifier & Fresh Interface
- **Automated Update Checker**: Checks GitHub Releases for new launcher versions and displays notifications with direct download links.
- **Streamlined Navigation**:
  - Tab bar: **Mod**, **Skins**, **Settings**, **Console**, **About**.
- **Dedicated About Page**:
  - Developed by **SYN 606** ([https://syn606.wtf](https://syn606.wtf)).
  - Official Minecraft portal link: [https://digitalvigital.fun/minecraft](https://digitalvigital.fun/minecraft).
  - Official Community Discord: [https://discord.gg/flex-dv](https://discord.gg/flex-dv).
  - Cleaned up legacy SMP link references across the entire codebase.

---

### 💎 Visual Identity & Installer Overhaul
- **New Cube Logo**: High-resolution obsidian, ruby, and gold cube branding across all assets (16x16, 32x32, 48x48, 64x64, 128x128, 256x256, 512x512).
- **Windows PE Resource Fix**:
  - Embedded canonical **`1 ICON` (`MAINICON`)** and `32512 ICON` directly into the PE `.rsrc` table, ensuring Windows Explorer, taskbar, Alt-Tab, and desktop shortcuts display the custom logo instead of default framework icons.
- **Setup Installer Visual Overhaul**:
  - Custom branded 150×57 header bitmap (`header.bmp`) and 164×314 wizard sidebar (`wizard.bmp`) featuring the glowing cube logo and launcher insignia.
  - Setup executable (`DigitalVigital-Setup-1.0.0.exe`) and uninstaller fully branded with canonical multi-resolution `.ico`.

---

### 📦 Clean-Machine Compatibility & WebView2 Bundling
- **Native `WebView2Loader.dll` Bundled**:
  - Automatically ships `WebView2Loader.dll` alongside the executables, inside the NSIS installer (`DigitalVigital-Setup-1.0.0.exe`), and within the portable zip archive.
  - Guarantees seamless launching on clean Windows test machines without triggering `0xC0000135` ("WebView2Loader.dll was not found") or requiring manual DLL installations.

### 🛡️ Smart Crash Diagnostics & Mod Installation Verification
- **Bytecode & JVM Compatibility Engine**: Automatically decodes Minecraft launch crashes (such as `UnsupportedClassVersionError 65.0 vs 52.0`) and immediately presents 1-click solutions ("Open Java Settings") directly in the Play tab UI.
- **Accurate Mod Verification via Modrinth Project ID / UUID**: Verifies installed mods by exact Modrinth project ID, slug, and file pattern, preventing false positive "Installed" indicators (e.g. installing *Sodium* no longer falsely marks *Sodium Extra* or *Sodium Chloride* as installed).

---

### 🧠 Dynamic RAM Allocation & Smart Session Hints
- **Reliable Dynamic Memory Allocation & Profile Synchronization**:
  - The Settings RAM slider now immediately and reliably calculates and injects the effective `-Xmx` (maximum heap) and `-Xms` (initial heap) parameters directly into the Minecraft JVM launch command.
  - Added 1-click fast memory preset pills (**2 GB**, **4 GB**, **6 GB**, **8 GB**, **12 GB**, **16 GB**) in Settings for instantaneous switching.
  - Auto-saves instantly on slider release, synchronizes the active instance memory, and immediately persists the updated allocation to `instance.json` on disk.
  - Added direct 1-click **Quick RAM Allocation presets** (**2 GB**, **4 GB**, **6 GB**, **8 GB**) directly on the Minecraft Profiles tab cards with instant two-way sync to global settings and live GB display.
- **Intelligent Non-Fatal Exit vs Crash Diagnostics**:
  - Minecraft session termination is now categorized with granular severity (`is_fatal: bool`).
  - Normal window closures, clean exits, and routine game stops no longer display alarming red crash alerts.
  - Instead, the launcher surfaces a gentle, dismissible amber/blue **Session Notice** ("Minecraft Session Ended") with a 1-click **"View Logs"** button, reserving high-priority error cards strictly for genuine JVM fatal exceptions, bytecode mismatches, and native library crashes.
- **Responsive Layout Across All Display Sizes**:
  - Fully responsive layout architecture adapting from 720p/768p compact laptop displays to 4K ultra-wide monitors.
  - Fluid padding, multi-column grid scaling, and auto-scrolling sidebar navigation so no controls or buttons are ever cut off on smaller screens.

---

### 🎮 Discord Rich Presence (RPC) & Official Branding
- **Live Launcher & In-Game Status**:
  - Automatically activates on launcher boot: displays **"In Launcher (Idle)"** with launcher details and links.
  - Automatically transitions when Minecraft launches: displays **"Playing Minecraft <Version>"** with live elapsed playtime timers.
  - Automatically resets to idle when Minecraft exits or terminates.
- **Embedded Launcher Logo**:
  - Direct HTTPS resolution via Discord proxy (`https://raw.githubusercontent.com/SYN606/dv-client/main/assets/logo.png`) guaranteeing the obsidian/gold launcher logo renders on Discord user profiles without requiring manual Developer Portal asset key configuration.
  - Interactive profile buttons: **"Play Minecraft"** and **"Join Discord"**.
  - Respects privacy settings: customizable options for showing Minecraft version, modpack, elapsed time, or toggling Discord RPC entirely.

### 🛡️ SmartScreen & Authenticode Digital Signing
- **Authenticode Digital Signatures**:
  - All release executables (`DigitalVigitalLauncher.exe`, `dv-launcher-app.exe`, `DigitalVigital-Setup-1.0.0.exe`) are digitally signed with an official Authenticode signature and RFC 3161 DigiCert timestamp.
  - Windows file properties now display verified Publisher: **Digital Vigital**.
- **1-Click Certificate Installer (`Trust-DigitalVigital.bat`)**:
  - Bundled `DigitalVigital.cer` and `Trust-DigitalVigital.bat` in `dist/` and inside the portable zip package.
  - Running `Trust-DigitalVigital.bat` as Administrator permanently adds the certificate to the local Trusted Root/Trusted Publisher stores, eliminating SmartScreen warnings for internal testing teams.

---

## Downloads & Available Packages (v1.0.0)

| Package | Size | Description |
| :--- | :---: | :--- |
| [**`DigitalVigital-Setup-1.0.0.exe`**](https://github.com/SYN606/dv-client/releases/download/v1.0.0/DigitalVigital-Setup-1.0.0.exe) | **~10.6 MB** | **Windows NSIS Setup**: Authenticode-signed installer with custom drive selection, Start Menu & Desktop shortcuts, clean uninstaller, bundled WebView2Loader, and Add/Remove Programs registration. |
| [**`DigitalVigital-v1.0.0-windows-x64-portable.zip`**](https://github.com/SYN606/dv-client/releases/download/v1.0.0/DigitalVigital-v1.0.0-windows-x64-portable.zip) | **~14.7 MB** | **Portable Standalone Package**: Contains signed `DigitalVigitalLauncher.exe`, `WebView2Loader.dll`, `DigitalVigital.cer`, `Trust-DigitalVigital.bat`, assets, and documentation. No installation required. |
| [**`DigitalVigitalLauncher.exe`**](https://github.com/SYN606/dv-client/releases/download/v1.0.0/DigitalVigitalLauncher.exe) | **41 MB** | **Direct Executable**: Standalone Windows executable with embedded primary icon resource (`1 ICON`) and Authenticode signature. |

---

## Verified SHA-256 Checksums (v1.0.0)
```text
9f593f69acf5de12065f7b45131fa160a6534528c6f2166993760e360ed4a5ba  DigitalVigital-Setup-1.0.0.exe
8032b809327a5ddb57d57540df86a9639c9aadaded7a0dd66900c3ce9ce214e5  DigitalVigital-v1.0.0-windows-x64-portable.zip
624a00930c4e7373820acecb37eacb0330edd8a16e16ca40ae64faee9bb8b557  DigitalVigitalLauncher.exe
25cc8888b3fb731434fc2550c3f7ca244dcc3082ee53510310a338609a96fa8f  dv-launcher-app.exe
86545b66cdb0603bc26b626fb9ad610cb6e71f28d468f5ea66df23b03dda96d5  WebView2Loader.dll
```

---

## System Requirements
- **Operating System**: Windows 10 (1809+) or Windows 11 (64-bit)
- **Webview Runtime**: Microsoft Edge WebView2 (preinstalled on modern Windows)
- **Memory**: 4 GB RAM minimum (8 GB+ recommended for modded Minecraft)
- **Storage**: 200 MB for launcher base installation; user-configurable drive for instances & cache
