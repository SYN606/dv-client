# Digital Vigital (DV) Launcher (Rust Engine + Slint Native Desktop UI)

> *Your Minecraft. Your Control.*

**Digital Vigital (DV) Launcher** is an independently engineered, modern, privacy-first desktop Minecraft launcher built in **Rust** and powered by the **Slint** native UI framework.

It completely eliminates Python runtime dependencies from production, replaces vulnerable static XOR credential masking with operating-system-backed hardware keyrings, rejects invasive telemetry, and provides segregated multi-instance management connected directly to official Minecraft and mod-loader infrastructure.

---

## ⚡ Core Highlights

- **Pure Rust Engine & Native Slint GUI**: Zero Electron, zero WebViews, zero Python runtime dependency in production. Fast, lightweight, and memory-safe.
- **Hardware & OS-Backed Credential Security**: Microsoft OAuth refresh tokens are stored exclusively via OS keyring facilities (`keyring-rs`):
  - **Linux**: Secret Service API via DBus (`zbus`)
  - **Windows**: Windows Credential Manager / DPAPI
  - **macOS**: Apple Keychain Services
- **Strict Clean-Room Architecture**: Zero proprietary launcher code, zero runtime bytecode instrumentation agents (`-javaagent`), and zero reverse-engineered decompiled logic.
- **Zero Telemetry**: No Google Analytics, no session telemetry, no machine fingerprinting, and zero unsolicited network pings.
- **High-Performance Verified Downloader**: Bounded concurrent transfers over HTTP/2 with in-flight streaming SHA-1 and SHA-256 validation against official Mojang catalogs.
- **Zip Slip Path Traversal Protection**: Enforces canonical destination boundaries on all extracted library archives and modpacks.
- **Multi-Instance Segregation**: Isolated `.minecraft` working directories, custom memory allocations, and version overrides.
- **Real-Time Sanitized Terminal**: Non-shell subprocess execution with live streaming and automatic in-flight redaction of OAuth bearer tokens and sensitive parameters.

---

## 🚀 Quick Start (Fedora Linux & Windows)

### Prerequisites

- **Rust toolchain (1.88+)** (e.g. `rustc 1.99.1+`, `cargo`)
- **Java Runtime Environment (JRE/JDK 21 Recommended)** for modern Minecraft versions
- Standard development packages (on Fedora: `mesa-libGL-devel`, `fontconfig-devel`, `openssl-devel`)

### Building & Running

```bash
# Check compilation
cargo check

# Run test suite
cargo test

# Launch Digital Vigital Launcher native desktop window
cargo run --release
```

---

## 🏗️ Architecture Overview

```text
+-------------------------------------------------------------------------+
| Digital Vigital Native Desktop UI (Slint)                               |
| - Obsidian Black & Crimson Red Design Tokens                            |
| - Play Dashboard, Profiles Manager, Accounts Vault, Downloads, Console  |
+------------------------------------+------------------------------------+
                                     | slint::invoke_from_event_loop
                                     v
+-------------------------------------------------------------------------+
| Digital Vigital Core Subsystems (Rust Engine)                           |
| - ConfigManager: Schema-validated JSON persistence & XDG/APPDATA paths  |
| - CredentialVault: OS-Backed Keyring (Linux Secret Service / DPAPI)     |
| - PistonMetaClient: Official Mojang Piston metadata & asset catalogs    |
| - AsyncDownloader: Bounded concurrent HTTP/2 streams + SHA-1 validation|
| - SafeExtractor: Zip Slip path traversal prevention engine              |
| - LaunchArgumentBuilder: Classpath composer & parameter interpolator    |
| - ProcessSupervisor: Subprocess orchestrator & in-flight secret scrubber|
+-------------------------------------------------------------------------+
```

---

## ⚖️ Legal & Trademarks

Minecraft is a registered trademark of Mojang Synergies AB and Microsoft Corporation. Digital Vigital Launcher is an independent clean-room open-source project and is **not affiliated with, endorsed by, or sponsored by Mojang Studios or Microsoft Corporation**.
