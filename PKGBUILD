# Maintainer: Sisyphus <sigmachan@users.noreply.github.com>
# Contributor: Sisyphus

pkgname=nsz-converter
pkgver=1.0.0
pkgrel=1
pkgdesc="Production-ready NSZ to NSP converter with auto-monitoring and beautiful GUI"
arch=('x86_64' 'aarch64')
url="https://github.com/sigmachan/nsz-converter"
license=('MIT')
depends=(
  'webkit2gtk-4.1'
  'gtk3'
  'libayatana-appindicator'
  'openssl'
  'zstd'
)
makedepends=(
  'cargo'
  'nodejs'
  'npm'
  'rust'
  'git'
  'base-devel'
  'appmenu-gtk-module'
)
optdepends=(
  'nsz: System nsz CLI (optional, app bundles prebuilt runtime)'
  'python: For system nsz installation'
)
provides=('nsz-converter')
conflicts=('nsz-converter-bin' 'nsz-converter-git')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/sigmachan/nsz-converter/archive/v${pkgver}.tar.gz")
sha256sums=('SKIP')  # Update with actual checksum for release

prepare() {
  cd "${srcdir}/${pkgname}-${pkgver}"
  
  # Fetch cargo dependencies
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
  cd "${srcdir}/${pkgname}-${pkgver}"
  
  # Build frontend
  cd frontend
  npm ci --prefer-offline --no-audit --no-fund
  npm run build
  
  cd ..
  
  # Build Tauri app (release mode)
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo tauri build --bundles deb
}

check() {
  cd "${srcdir}/${pkgname}-${pkgver}"
  export RUSTUP_TOOLCHAIN=stable
  cargo test --frozen --all-features
}

package() {
  cd "${srcdir}/${pkgname}-${pkgver}"
  
  # Install binary
  install -Dm755 "src-tauri/target/release/nsz-converter" \
    "${pkgdir}/usr/bin/nsz-converter"
  
  # Install desktop file (create if not present)
  install -Dm644 /dev/stdin "${pkgdir}/usr/share/applications/nsz-converter.desktop" << 'DESKTOP_EOF'
[Desktop Entry]
Type=Application
Version=1.0
Name=NSZ Converter
Comment=Production-ready NSZ to NSP converter with auto-monitoring
Exec=nsz-converter
Icon=nsz-converter
Terminal=false
Categories=Utility;Game;
StartupNotify=true
DESKTOP_EOF
  
  # Install icons (create placeholder - user should add actual icons)
  for size in 16 32 48 64 128 256 512; do
    install -dm755 "${pkgdir}/usr/share/icons/hicolor/${size}x${size}/apps"
    # Placeholder - copy actual icon files here
    # install -Dm644 "icons/${size}x${size}.png" \
    #   "${pkgdir}/usr/share/icons/hicolor/${size}x${size}/apps/nsz-converter.png"
  done
  
  # Install AppImage if built (alternative distribution method)
  if [ -f "src-tauri/target/release/bundle/appimage/nsz-converter_${pkgver}_amd64.AppImage" ]; then
    install -Dm755 "src-tauri/target/release/bundle/appimage/nsz-converter_${pkgver}_amd64.AppImage" \
      "${pkgdir}/usr/bin/nsz-converter-appimage"
  fi
  
  # Install documentation
  install -Dm644 README.md "${pkgdir}/usr/share/doc/${pkgname}/README.md"
  install -Dm644 ARCHITECTURE.md "${pkgdir}/usr/share/doc/${pkgname}/ARCHITECTURE.md" 2>/dev/null || true
  install -Dm644 BUILD.md "${pkgdir}/usr/share/doc/${pkgname}/BUILD.md" 2>/dev/null || true
  
  # Install license
  install -Dm644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE" 2>/dev/null || true
  
  # Install systemd user service (optional - for background monitoring)
  install -Dm644 /dev/stdin "${pkgdir}/usr/lib/systemd/user/nsz-converter.service" << 'SERVICE_EOF'
[Unit]
Description=NSZ Converter Background Monitor
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/nsz-converter --background
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
SERVICE_EOF
}

# CachyOS specific optimizations (optional)
# Uncomment if building specifically for CachyOS
# options=(!lto)  # CachyOS uses LTO by default in makepkg.conf
