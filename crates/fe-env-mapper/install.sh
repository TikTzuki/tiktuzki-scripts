#!/bin/bash
set -e

echo ''
echo '  _______ _____ _  __'
echo ' |__   __|_   _| |/ /'
echo '    | |    | | | ''  / '
echo '    | |    | | |  <  '
echo '    | |   _| |_| . \ '
echo '    |_|  |_____|_|\_\'
echo ''
echo '             TIK TIKTUZKI'
echo ''
echo '------------------------------------------'
echo '      Welcome to Tik'"'"'s Script Engine     '
echo '------------------------------------------'
echo ''
echo 'Now attempting installation...'
echo ''

TOOL=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tool)
      TOOL="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

# Global variables
export MAP_ENV_SERVICE="https://github.com/TikTzuki/tiktuzki-scripts"

if [ -z "$MAP_ENV_DIR" ]; then
    MAP_ENV_DIR="$HOME/.map_env"
    MAP_ENV_DIR_RAW='$HOME/.map_env'
else
    MAP_ENV_DIR_RAW="$MAP_ENV_DIR"
fi
export MAP_ENV_DIR

# Local variables
map_env_bin_folder="${MAP_ENV_DIR}/bin"
map_env_config_file="${MAP_ENV_DIR}/config"
map_env_bash_profile="${HOME}/.bash_profile"
map_env_bashrc="${HOME}/.bashrc"
map_env_zshrc="${ZDOTDIR:-${HOME}}/.zshrc"

map_env_init_snippet=$( cat << EOF
# MAP_ENV
export MAP_ENV_DIR="$MAP_ENV_DIR_RAW"
export PATH="\$MAP_ENV_DIR/bin:\$PATH"
EOF
)

# OS detection
darwin=false
case "$(uname)" in
    Darwin*) darwin=true ;;
esac

echo "Looking for a previous installation of map_env..."
if [ -d "$MAP_ENV_DIR" ]; then
    echo "map_env found at ${MAP_ENV_DIR}"
    echo "Please remove existing installation first: rm -rf ${MAP_ENV_DIR}"
    exit 0
fi

# Check dependencies
echo "Checking dependencies..."
for cmd in curl tar; do
    if ! command -v "$cmd" > /dev/null; then
        echo "Error: $cmd is required but not installed."
        exit 1
    fi
done

# Create directories
echo "Creating directories..."
mkdir -p "$map_env_bin_folder"

# Download and install binary
echo "Prime platform file..."
# infer platform
function infer_platform() {
	local kernel
	local machine

	kernel="$(uname -s)"
	machine="$(uname -m)"

	case $kernel in
	Linux)
	  case $machine in
	  i686)
		echo "linuxx32"
		;;
	  x86_64)
		echo "x86_64-unknown-linux-gnu"
		;;
	  armv6l)
		echo "linuxarm32hf"
		;;
	  armv7l)
		echo "linuxarm32hf"
		;;
	  armv8l)
		echo "linuxarm32hf"
		;;
	  aarch64)
		echo "aarch64-unknown-linux-gnu"
		;;
	  *)
	  	echo "x86_64-unknown-linux-gnu"
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
	MSYS*|MINGW*)
	  case $machine in
	  x86_64)
		echo "windowsx64"
		;;
	  *)
	  	echo "exotic"
	  	;;
	  esac
	  ;;
	*)
	  echo "exotic"
	esac
}

export PLATFORM="$(infer_platform)"

BINARY_URL="${MAP_ENV_SERVICE}/releases/latest/download/env-mapper-${PLATFORM}"
echo "Downloading from $BINARY_URL"
curl -L "$BINARY_URL" -o "${map_env_bin_folder}/map_env"
echo "Map env installed at ${map_env_bin_folder}/map_env"
chmod +x "${map_env_bin_folder}/map_env"

# Update shell profiles
if [[ $darwin == true ]]; then
    touch "$map_env_bash_profile"
    if [[ -z $(grep 'MAP_ENV_DIR' "$map_env_bash_profile") ]]; then
        echo -e "\n$map_env_init_snippet" >> "$map_env_bash_profile"
        echo "Added map_env to $map_env_bash_profile"
    fi
else
    touch "$map_env_bashrc"
    if [[ -z $(grep 'MAP_ENV_DIR' "$map_env_bashrc") ]]; then
        echo -e "\n$map_env_init_snippet" >> "$map_env_bashrc"
        echo "Added map_env to $map_env_bashrc"
    fi
fi

touch "$map_env_zshrc"
if [[ -z $(grep 'MAP_ENV_DIR' "$map_env_zshrc") ]]; then
    echo -e "\n$map_env_init_snippet" >> "$map_env_zshrc"
    echo "Added map_env to $map_env_zshrc"
fi

echo "Installation complete!"
echo "Please restart your terminal or run:"
echo "    source ~/.bashrc  # or ~/.zshrc"
echo "Then try: map_env --help"
