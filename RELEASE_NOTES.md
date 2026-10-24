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
- **Automatic Version Resolution**:
  - Minecraft ≤ 1.16.5: Automatically routes to **Java 8** (essential for Forge 1.8.9 / 1.12.2).
  - Minecraft 1.17 – 1.20.4: Automatically routes to **Java 17**.
  - Minecraft 1.20.5+: Automatically routes to **Java 21**.
- **Automated 1-Click Java Installers**: Download and install Temurin OpenJDK runtimes directly into isolated launcher directories without requiring administrative rights.
- **Custom Java Path Support**: Browse and select custom JVM installations with system architecture validation.

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

### 💎 Visual Identity & Canonical Windows Resource Icon
- **New Cube Logo**: High-resolution obsidian, ruby, and gold cube branding across all assets (16x16, 32x32, 48x48, 64x64, 128x128, 256x256, 512x512).
- **Windows PE Resource Fix**:
  - Embedded canonical **`1 ICON` (`MAINICON`)** and `32512 ICON` directly into the PE `.rsrc` table, ensuring Windows Explorer, taskbar, Alt-Tab, and desktop shortcuts display the custom logo instead of default framework icons.

---

## Downloads & Available Packages (v1.0.0)

| Package | Size | Description |
| :--- | :---: | :--- |
| [**`DigitalVigital-Setup-1.0.0.exe`**](https://github.com/SYN606/dv-client/releases/download/v1.0.0/DigitalVigital-Setup-1.0.0.exe) | **~10.5 MB** | **Windows NSIS Setup**: Interactive installer with custom drive selection, Start Menu & Desktop shortcuts, clean uninstaller, and Add/Remove Programs registration. |
| [**`DigitalVigital-v1.0.0-windows-x64-portable.zip`**](https://github.com/SYN606/dv-client/releases/download/v1.0.0/DigitalVigital-v1.0.0-windows-x64-portable.zip) | **~28.5 MB** | **Portable Standalone Package**: Contains `DigitalVigitalLauncher.exe`, `dv-launcher-app.exe`, assets, and documentation. No installation required. |
| [**`DigitalVigitalLauncher.exe`**](https://github.com/SYN606/dv-client/releases/download/v1.0.0/DigitalVigitalLauncher.exe) | **41 MB** | **Direct Executable**: Standalone Windows executable with embedded primary icon resource (`1 ICON`). |

---

## Verified SHA-256 Checksums (v1.0.0)
```text
654a7740ace6e2b84362c03ca8ad0a30580eb4ac93caae382a018f09de3dd669  DigitalVigital-Setup-1.0.0.exe
095a1fc4506466cc772c44fe6e7a19af628a55377cab941b648d29440a2f14fa  DigitalVigital-v1.0.0-windows-x64-portable.zip
5937d640ac762cb234d93bbc529e5c9cfd7c2b18bfed886f01e5a1cea152a3c7  DigitalVigitalLauncher.exe
5937d640ac762cb234d93bbc529e5c9cfd7c2b18bfed886f01e5a1cea152a3c7  dv-launcher-app.exe
```

---

## System Requirements
- **Operating System**: Windows 10 (1809+) or Windows 11 (64-bit)
- **Webview Runtime**: Microsoft Edge WebView2 (preinstalled on modern Windows)
- **Memory**: 4 GB RAM minimum (8 GB+ recommended for modded Minecraft)
- **Storage**: 200 MB for launcher base installation; user-configurable drive for instances & cache
