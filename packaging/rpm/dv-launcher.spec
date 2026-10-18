Name:           dv-launcher
Version:        1.0.0
Release:        1%{?dist}
Summary:        High-performance, minimal, and privacy-first Minecraft launcher in native Rust
License:        Apache-2.0
URL:            https://digitalvigital.com
BuildArch:      x86_64

%description
Digital Vigital (DV) is a high-performance native Minecraft launcher written
in 100% safe Rust with a modern Slint UI. Featuring zero tracking, single-instance
RAM minimization, isolated modpack profiles, and instant launch capabilities.

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/512x512/apps
mkdir -p %{buildroot}%{_datadir}/pixmaps

install -m 755 %{_sourcedir}/dv-launcher %{buildroot}%{_bindir}/dv-launcher
install -m 644 %{_sourcedir}/dv-launcher.desktop %{buildroot}%{_datadir}/applications/dv-launcher.desktop
install -m 644 %{_sourcedir}/logo.png %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/dv-launcher.png
install -m 644 %{_sourcedir}/logo.png %{buildroot}%{_datadir}/pixmaps/dv-launcher.png

%files
%{_bindir}/dv-launcher
%{_datadir}/applications/dv-launcher.desktop
%{_datadir}/icons/hicolor/512x512/apps/dv-launcher.png
%{_datadir}/pixmaps/dv-launcher.png

%changelog
* Sun Oct 18 2026 Digital Vigital <contact@digitalvigital.com> - 1.0.0-1
- Initial production release of Digital Vigital native Minecraft launcher
