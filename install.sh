#!/bin/bash
set -e

function infer_platform() {
  local kernel="$(uname -s)"
  local machine="$(uname -m)"

	case $kernel in
	Linux)
	  case $machine in
	  x86_64)
		echo "x86_64-unknown-linux-musl"
		;;
	  aarch64)
		echo "aarch64-unknown-linux-musl"
		;;
	  *)
	  	echo "x86_64-unknown-linux-musl"
	  	;;
	  esac
	  ;;
	Darwin)
	  case $machine in
	  x86_64)
		echo "x86_64-apple-darwin"
		;;
	  arm64)
		echo "aarch64-apple-darwin"
		;;
	  *)
	    echo "x86_64-apple-darwin"
	    ;;
	    esac
	  ;;
	*)
	  echo "unsupported"
	esac
}

export PLATFORM=$(infer_platform)

if [ "$PLATFORM" = "unsupported" ]; then
  echo "Error: Unsupported platform"
  exit 1
fi

# GitHub release info
BINARY_NAME="installer-${PLATFORM}"
DOWNLOAD_URL="https://github.com/TikTzuki/tiktuzki-scripts/releases/latest/download/${BINARY_NAME}"

# Download binary
echo "Downloading installer for ${PLATFORM}..."
TEMP_BIN="/tmp/tiks-installer"
curl -L "$DOWNLOAD_URL" -o "$TEMP_BIN"

# Make executable and run
chmod +x "$TEMP_BIN"
echo "Running installer..."
"$TEMP_BIN" "$@"

# Cleanup
rm -f "$TEMP_BIN"
