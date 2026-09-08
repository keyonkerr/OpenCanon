#!/bin/sh
# Install the latest OpenCanon CLI binary from GitHub Releases.
# Usage: curl -fsSL https://raw.githubusercontent.com/keyonkerr/OpenCanon/master/scripts/install.sh | sh
set -eu

REPO="${OPENCANON_REPO:-keyonkerr/OpenCanon}"
BIN="opencanon"
BIN_DIR="${OPENCANON_INSTALL_DIR:-$HOME/.local/bin}"
RELEASE="${OPENCANON_RELEASE:-latest}"

die() {
  printf 'install.sh: %s\n' "$1" >&2
  exit 1
}

step() {
  printf '==> %s\n' "$1"
}

normalize_release() {
  case "$1" in
    latest | "") printf 'latest\n' ;;
    v*) printf '%s\n' "$1" ;;
    *) printf 'v%s\n' "$1" ;;
  esac
}

detect_target() {
  os=$(uname -s)
  arch=$(uname -m)
  case "$os" in
    Darwin)
      case "$arch" in
        arm64 | aarch64) printf 'aarch64-apple-darwin\n' ;;
        x86_64) printf 'x86_64-apple-darwin\n' ;;
        *) die "unsupported macOS architecture: $arch" ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64 | amd64) printf 'x86_64-unknown-linux-musl\n' ;;
        aarch64 | arm64) printf 'aarch64-unknown-linux-musl\n' ;;
        *) die "unsupported Linux architecture: $arch" ;;
      esac
      ;;
    MINGW* | MSYS* | CYGWIN*)
      die "on Windows use: irm https://raw.githubusercontent.com/${REPO}/master/scripts/install.ps1 | iex"
      ;;
    *)
      die "unsupported OS: $os"
      ;;
  esac
}

asset_url() {
  asset="$1"
  if [ "$RELEASE" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download/%s\n' "$REPO" "$asset"
  else
    printf 'https://github.com/%s/releases/download/%s/%s\n' "$REPO" "$RELEASE" "$asset"
  fi
}

download() {
  url="$1"
  dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest" || return 1
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$dest" "$url" || return 1
  else
    die "need curl or wget"
  fi
}

file_sha256() {
  file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
  else
    die "need sha256sum or shasum to verify the download"
  fi
}

RELEASE=$(normalize_release "$RELEASE")
TARGET=$(detect_target)
ARCHIVE="${BIN}-${TARGET}.tar.gz"
CHECKSUM="${BIN}-${TARGET}.sha256"

step "OpenCanon CLI"
step "platform: $TARGET"
step "release: $RELEASE"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

ARCHIVE_PATH="$TMP/$ARCHIVE"
CHECKSUM_PATH="$TMP/$CHECKSUM"

if ! download "$(asset_url "$ARCHIVE")" "$ARCHIVE_PATH"; then
  die "could not download $ARCHIVE from https://github.com/${REPO}/releases (has a tagged GitHub Release been published?)"
fi
if ! download "$(asset_url "$CHECKSUM")" "$CHECKSUM_PATH"; then
  die "could not download $CHECKSUM (release is missing checksums)"
fi

expected=$(awk '{print $1; exit}' "$CHECKSUM_PATH")
actual=$(file_sha256 "$ARCHIVE_PATH")
if [ "$expected" != "$actual" ]; then
  die "SHA-256 mismatch for $ARCHIVE"
fi

tar -xzf "$ARCHIVE_PATH" -C "$TMP"
if [ ! -f "$TMP/$BIN" ]; then
  die "archive did not contain $BIN"
fi

mkdir -p "$BIN_DIR"
cp "$TMP/$BIN" "$BIN_DIR/$BIN"
chmod 755 "$BIN_DIR/$BIN"

step "installed $BIN_DIR/$BIN"

case ":$PATH:" in
  *":$BIN_DIR:"*)
    step "$BIN_DIR is already on PATH"
    ;;
  *)
    printf 'WARNING: %s is not on PATH. Add this to your shell profile:\n  export PATH="%s:$PATH"\n' "$BIN_DIR" "$BIN_DIR" >&2
    ;;
esac

step "run: opencanon help"
