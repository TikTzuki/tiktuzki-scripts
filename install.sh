#!/bin/bash
set -e

# Global variables
export MAP_ENV_VERSION="1.0.0"
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
    echo "Please remove existing installation first."
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
echo "Downloading map_env..."
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

BINARY_URL="${MAP_ENV_SERVICE}/releases/latest/download/fe-map_env-${PLATFORM}-${ARCH}"
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
