Name:           krebs
Version:        0.1.0
Release:        1%{?dist}
Summary:        Minimalist Writing & Infinite Whiteboard Studio in Pure Rust
License:        MIT
URL:            https://github.com/ybreakless/Krebs
Source0:        https://github.com/ybreakless/Krebs/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  fontconfig-devel
BuildRequires:  libX11-devel
BuildRequires:  libXcursor-devel
BuildRequires:  libXrandr-devel
BuildRequires:  libXi-devel
BuildRequires:  libXinerama-devel
BuildRequires:  libxkbcommon-devel
BuildRequires:  mesa-libGL-devel
BuildRequires:  wayland-devel

Requires:       fontconfig
Requires:       libX11
Requires:       libXcursor
Requires:       libXrandr
Requires:       libXi
Requires:       libXinerama
Requires:       libxkbcommon
Requires:       mesa-libGL
Requires:       wayland

%description
Krebs is an ultra-minimalist, distraction-free writing studio and infinite
vector whiteboard built in 100% pure Rust. Features first-class trilingual
support (English, German, Persian/Farsi with HarfBuzz/swash cursive shaping),
exam countdown timers, an integrated Hermes AI essay coach, and complete
portability.

%prep
%autosetup -n Krebs-%{version}

%build
cargo build --release --locked

%install
rm -rf $RPM_BUILD_ROOT
install -d $RPM_BUILD_ROOT%{_bindir}
install -d $RPM_BUILD_ROOT%{_datadir}/applications
install -d $RPM_BUILD_ROOT%{_datadir}/icons/hicolor/scalable/apps
install -d $RPM_BUILD_ROOT%{_datadir}/icons/hicolor/512x512/apps
install -d $RPM_BUILD_ROOT%{_docdir}/%{name}
install -d $RPM_BUILD_ROOT%{_licensesdir}/%{name}

install -m 755 target/release/krebs $RPM_BUILD_ROOT%{_bindir}/krebs
install -m 644 packaging/linux/krebs.desktop $RPM_BUILD_ROOT%{_datadir}/applications/krebs.desktop
install -m 644 assets/logo.svg $RPM_BUILD_ROOT%{_datadir}/icons/hicolor/scalable/apps/krebs.svg
install -m 644 assets/logo.png $RPM_BUILD_ROOT%{_datadir}/icons/hicolor/512x512/apps/krebs.png
install -m 644 README.md $RPM_BUILD_ROOT%{_docdir}/%{name}/README.md
install -m 644 LICENSE $RPM_BUILD_ROOT%{_licensesdir}/%{name}/LICENSE

%files
%license LICENSE
%doc README.md
%{_bindir}/krebs
%{_datadir}/applications/krebs.desktop
%{_datadir}/icons/hicolor/scalable/apps/krebs.svg
%{_datadir}/icons/hicolor/512x512/apps/krebs.png

%changelog
* Wed Oct 28 2026 ybreakless <194576839+ybreakless@users.noreply.github.com> - 0.1.0-1
- Initial release of Krebs for Linux (RPM/DNF).
