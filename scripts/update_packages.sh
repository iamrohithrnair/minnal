#!/usr/bin/env bash
# Update Homebrew tap and AUR package for a new release.
# Usage: scripts/update_packages.sh v0.1.3
set -euo pipefail

VERSION="${1:?Usage: $0 <version-tag>}"
VERSION_NUM="${VERSION#v}"

echo "Updating packages for $VERSION..."

LINUX_URL="https://github.com/codeslord/minnal/releases/download/${VERSION}/minnal-linux-x86_64.tar.gz"
LINUX_ARM_URL="https://github.com/codeslord/minnal/releases/download/${VERSION}/minnal-linux-aarch64.tar.gz"
MACOS_ARM_URL="https://github.com/codeslord/minnal/releases/download/${VERSION}/minnal-macos-aarch64.tar.gz"

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

echo "Downloading assets for checksums..."
curl -sL "$LINUX_URL" -o "$tmpdir/linux.tar.gz"
curl -sL "$LINUX_ARM_URL" -o "$tmpdir/linux-arm.tar.gz"
curl -sL "$MACOS_ARM_URL" -o "$tmpdir/macos-arm.tar.gz"

LINUX_SHA=$(sha256sum "$tmpdir/linux.tar.gz" | cut -d' ' -f1)
LINUX_ARM_SHA=$(sha256sum "$tmpdir/linux-arm.tar.gz" | cut -d' ' -f1)
MACOS_ARM_SHA=$(sha256sum "$tmpdir/macos-arm.tar.gz" | cut -d' ' -f1)

  echo "  Linux SHA256: $LINUX_SHA"
echo "  Linux ARM64 SHA256: $LINUX_ARM_SHA"
echo "  macOS ARM64 SHA256: $MACOS_ARM_SHA"

# --- Homebrew tap ---
echo ""
echo "Updating Homebrew tap..."
BREW_DIR="$tmpdir/homebrew-minnal"
git clone --depth 1 git@github.com:codeslord/homebrew-minnal.git "$BREW_DIR" 2>/dev/null

cat > "$BREW_DIR/Formula/minnal.rb" <<EOF
class Minnal < Formula
  desc "AI coding agent powered by Claude and ChatGPT"
  homepage "https://github.com/codeslord/minnal"
  version "$VERSION_NUM"
  license "MIT"

  on_macos do
    on_arm do
      url "$MACOS_ARM_URL"
      sha256 "$MACOS_ARM_SHA"

      def install
        bin.install "minnal-macos-aarch64" => "minnal"
      end
    end

  end

  on_linux do
    on_intel do
      url "$LINUX_URL"
      sha256 "$LINUX_SHA"

      def install
        libexec.install "minnal-linux-x86_64", "minnal-linux-x86_64.bin"
        libexec.install Dir["libssl.so*"], Dir["libcrypto.so*"]
        (bin/"minnal").write <<~SH
          #!/bin/sh
          exec "#{libexec}/minnal-linux-x86_64" "\$@"
        SH
      end
    end

    on_arm do
      url "$LINUX_ARM_URL"
      sha256 "$LINUX_ARM_SHA"

      def install
        bin.install "minnal-linux-aarch64" => "minnal"
      end
    end
  end

  test do
    assert_match "minnal", shell_output("#{bin}/minnal --version")
  end
end
EOF

(cd "$BREW_DIR" && git add -A && git commit -m "Update minnal to $VERSION" && git push origin main)
echo "  ✅ Homebrew tap updated"

# --- AUR ---
echo ""
echo "Updating AUR package..."
AUR_DIR="$tmpdir/minnal-bin-aur"
git clone ssh://aur@aur.archlinux.org/minnal-bin.git "$AUR_DIR" 2>/dev/null

cat > "$AUR_DIR/PKGBUILD" <<EOF
# Maintainer: Jeremy Huang <jeremyhuang55555@gmail.com>
pkgname=minnal-bin
pkgver=$VERSION_NUM
pkgrel=1
pkgdesc="AI coding agent powered by Claude and ChatGPT"
arch=('x86_64')
url="https://github.com/codeslord/minnal"
license=('MIT')
provides=('minnal')
conflicts=('minnal')
source=("$LINUX_URL")
sha256sums=('$LINUX_SHA')

package() {
    install -Dm755 "\${srcdir}/minnal-linux-x86_64" "\${pkgdir}/usr/lib/minnal/minnal-linux-x86_64"
    install -Dm755 "\${srcdir}/minnal-linux-x86_64.bin" "\${pkgdir}/usr/lib/minnal/minnal-linux-x86_64.bin"
    install -Dm644 "\${srcdir}"/libssl.so* "\${pkgdir}/usr/lib/minnal/"
    install -Dm644 "\${srcdir}"/libcrypto.so* "\${pkgdir}/usr/lib/minnal/"
    mkdir -p "\${pkgdir}/usr/bin"
    ln -s /usr/lib/minnal/minnal-linux-x86_64 "\${pkgdir}/usr/bin/minnal"
}
EOF

(cd "$AUR_DIR" && makepkg --printsrcinfo > .SRCINFO && git add -A && git commit -m "Update to $VERSION" && git push origin master)
echo "  ✅ AUR package updated"

echo ""
echo "Done! Packages updated to $VERSION"
