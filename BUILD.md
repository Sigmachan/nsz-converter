# NSZ Converter - Production Build Guide

## Building for All Platforms

This guide covers creating production distributables:
- **macOS**: `.app` bundle + `.dmg` installer
- **Windows**: `.exe` installer (MSI)
- **Linux**: `.AppImage` + `.deb`

---

## Prerequisites

### All Platforms

```bash
# Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js 18+ (LTS recommended)
# https://nodejs.org/

# Tauri CLI
cargo install tauri-cli@2
```

### Platform-Specific

**macOS** (for code signing, optional but recommended):
```bash
# Install Xcode Command Line Tools
xcode-select --install

# For distribution outside App Store, you'll need:
# - Apple Developer Program membership ($99/year)
# - Developer ID Application certificate
```

**Windows**:
- Visual Studio Build Tools (or full VS 2022)
- WiX Toolset v3.11+ (for MSI installer)

**Linux**:
- `webkit2gtk-4.1` development files
- `build-essential`, `curl`, `wget`, `file`, `libxdo-dev`

---

## Build Commands

### Development (Hot Reload)

```bash
cd nsz-converter
cargo tauri dev
```

### Production Builds

From project root:

```bash
# macOS (requires macOS host)
cargo tauri build --target aarch64-apple-darwin   # Apple Silicon
cargo tauri build --target x86_64-apple-darwin    # Intel

# Windows (requires Windows host or cross-compilation)
cargo tauri build --target x86_64-pc-windows-msvc

# Linux (requires Linux host)
cargo tauri build --target x86_64-unknown-linux-gnu
cargo tauri build --target aarch64-unknown-linux-gnu
```

**Output locations:**
```
src-tauri/target/release/bundle/
├── macos/
│   └── NSZ Converter.app
├── dmg/
│   └── NSZ Converter_1.0.0_aarch64.dmg
├── msi/
│   └── NSZ Converter_1.0.0_x64_en-US.msi
├── appimage/
│   └── nsz-converter_1.0.0_amd64.AppImage
└── deb/
    └── nsz-converter_1.0.0_amd64.deb
```

---

## macOS Distribution

### 1. Build the App Bundle

```bash
cargo tauri build --target aarch64-apple-darwin
```

Produces: `NSZ Converter.app`

### 2. Create DMG (Automatic)

Tauri automatically creates a `.dmg` when you run `tauri build`.

### 3. Code Signing (Recommended)

Create `src-tauri/entitlements.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.allow-dyld-share-cache</key>
    <true/>
    <key>com.apple.security.cs.debugger</key>
    <true/>
</dict>
</plist>
```

Update `tauri.conf.json`:

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAMID)",
      "entitlements": "entitlements.plist"
    }
  }
}
```

### 4. Notarization (Required for macOS 10.15+)

```bash
# After building signed .app
xcrun notarytool submit \
  "src-tauri/target/release/bundle/dmg/NSZ Converter_1.0.0_aarch64.dmg" \
  --apple-id your@email.com \
  --team-id YOURTEAMID \
  --password @env:APP_SPECIFIC_PASSWORD
```

---

## Windows Distribution

### Build MSI Installer

```bash
cargo tauri build --target x86_64-pc-windows-msvc
```

Produces:
- `NSZ Converter_1.0.0_x64_en-US.msi` (recommended for distribution)
- `nsz-converter.exe` (portable)

### Code Signing (Optional but Recommended)

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

---

## Linux Distribution

### AppImage (Recommended)

AppImage is the most user-friendly format - single file, runs everywhere.

```bash
cargo tauri build --target x86_64-unknown-linux-gnu
```

Output: `nsz-converter_1.0.0_amd64.AppImage`

**Usage:**
```bash
chmod +x nsz-converter_1.0.0_amd64.AppImage
./nsz-converter_1.0.0_amd64.AppImage
```

### Debian Package

```bash
cargo tauri build --target x86_64-unknown-linux-gnu
```

Output: `nsz-converter_1.0.0_amd64.deb`

**Install:**
```bash
sudo dpkg -i nsz-converter_1.0.0_amd64.deb
```

### AppImage Best Practices

1. **Include all dependencies** - Tauri bundles everything needed
2. **Test on multiple distros** - Ubuntu, Fedora, Arch
3. **Provide `.desktop` file** - Created automatically

---

## Embedded nsz Runtime

The app includes a `RuntimeManager` that:

1. **First checks** for system `nsz` installation
2. **Falls back** to bundled prebuilt binaries
3. **Auto-downloads** from GitHub releases if neither exists

### Bundled Binaries

Prebuilt `nsz` binaries are downloaded from:
https://github.com/nicoboss/nsz/releases

Supported platforms:
- `nsz-cli-macos` (arm64 + x64)
- `nsz-cli.exe` (Windows x64)
- `nsz-cli-linux` (x64 + arm64)

**Size:** ~15-25 MB per platform (compressed in installer)

---

## CI/CD Pipeline (GitHub Actions)

Create `.github/workflows/build.yml`:

```yaml
name: Build

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: macos-aarch64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: macos-x64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: windows-x64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: linux-x64

    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
      
      - name: Install dependencies (Linux)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev \
            build-essential curl wget file libxdo-dev
      
      - name: Install frontend deps
        run: cd frontend && npm install
      
      - name: Build
        run: cargo tauri build --target ${{ matrix.target }}
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: |
            src-tauri/target/${{ matrix.target }}/release/bundle/**
```

---

## Release Checklist

Before tagging a release:

- [ ] All tests pass on target platforms
- [ ] `nsz` runtime downloads and works on clean system
- [ ] File monitoring works (drag .nsz into monitored folder)
- [ ] Game mappings persist across restarts
- [ ] Conversion creates correct folder structure
- [ ] No console errors in production build
- [ ] App icons are crisp at all sizes
- [ ] Dark theme renders correctly
- [ ] macOS: App notarized and stapled
- [ ] Windows: MSI installs without errors
- [ ] Linux: AppImage runs on Ubuntu 22.04+

---

## Troubleshooting

### "nsz binary not found"

The app will auto-download. If it fails:
1. Manually install: `pip3 install nsz`
2. Or download from: https://github.com/nicoboss/nsz/releases

### macOS "App is damaged"

Solution: Right-click → Open, or run:
```bash
xattr -cr "/Applications/NSZ Converter.app"
```

### Windows "Windows protected your PC"

Click "More info" → "Run anyway" (or sign the binary)

### Linux "Permission denied"

```bash
chmod +x nsz-converter_*.AppImage
```

---

## File Sizes (Approximate)

| Platform | Installer | App Size | Notes |
|----------|-----------|----------|-------|
| macOS DMG | ~45 MB | ~120 MB | Includes nsz runtime |
| Windows MSI | ~35 MB | ~95 MB | Bundled runtime |
| Linux AppImage | ~40 MB | ~110 MB | Self-contained |
| Linux .deb | ~25 MB | ~85 MB | Requires system deps |

---

## Support

For issues with:
- **NSZ format/conversion**: https://github.com/nicoboss/nsz/issues
- **This app**: Create issue on your repo
