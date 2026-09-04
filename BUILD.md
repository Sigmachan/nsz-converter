# NSZ Converter - Build Guide

## Quick Start (macOS)

```bash
cd ~/dev/nsz-converter

# Install dependencies
cd frontend && npm install && cd ..

# Build the app
cargo tauri build

# Output: src-tauri/target/release/bundle/
```

## Prerequisites

### All Platforms
- Rust (latest stable): https://rustup.rs/
- Node.js 18+: https://nodejs.org/

### macOS
- Xcode Command Line Tools: `xcode-select --install`

### Windows
- Visual Studio Build Tools or VS 2022
- WiX Toolset v3.11+ (for MSI installer)

### Linux (Ubuntu/Debian)
```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libgtk-3-dev \
  pkg-config
```

## Build Commands

### Development (Hot Reload)
```bash
cargo tauri dev
```

### Production Build

#### macOS
```bash
cargo tauri build --target aarch64-apple-darwin  # Apple Silicon
cargo tauri build --target x86_64-apple-darwin   # Intel
```

#### Windows
```bash
cargo tauri build --target x86_64-pc-windows-msvc
```

#### Linux
```bash
cargo tauri build --target x86_64-unknown-linux-gnu
```

## Output Files

| Platform | File | Location |
|----------|------|----------|
| macOS | `.app` | `src-tauri/target/release/bundle/macos/` |
| macOS | `.dmg` | `src-tauri/target/release/bundle/dmg/` |
| Windows | `.exe` | `src-tauri/target/release/` |
| Windows | `.msi` | `src-tauri/target/release/bundle/msi/` |
| Linux | `.AppImage` | `src-tauri/target/release/bundle/appimage/` |
| Linux | `.deb` | `src-tauri/target/release/bundle/deb/` |

## Distribution

### macOS
1. Build the `.dmg`
2. Code sign (optional but recommended for distribution)
3. Notarize with Apple (required for macOS 10.15+)

### Windows
1. Build the `.msi`
2. Sign the installer (optional but recommended)
3. Distribute the `.msi` file

### Linux
1. Build the `.AppImage` (recommended - single file, runs everywhere)
2. Build the `.deb` (for Debian/Ubuntu users)

### CachyOS/Arch Linux
Use the provided `PKGBUILD`:
```bash
makepkg -si
```

## Embedded NSZ Runtime

The app includes a `RuntimeManager` that:
1. Checks for system `nsz` installation
2. Falls back to bundled prebuilt binaries
3. Auto-downloads from GitHub releases if needed

No Python installation required for end users!

## First Run Requirements

Users need to provide their own `prod.keys` file (Nintendo Switch keys).

Place in:
- macOS/Linux: `~/.switch/prod.keys`
- Windows: `%USERPROFILE%\.switch\prod.keys`

## Troubleshooting

### "nsz binary not found"
The app will auto-download. If it fails:
```bash
pip3 install nsz
```

### macOS "App is damaged"
```bash
xattr -cr "/Applications/NSZ Converter.app"
```

### Linux "Permission denied"
```bash
chmod +x nsz-converter*.AppImage
```
