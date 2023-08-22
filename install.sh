#!/bin/sh

get_platform() {
    case $(uname | tr '[:upper:]' '[:lower:]') in
        linux*)
            echo "unknown-linux-gnu"
            ;;
        darwin*)
            echo "apple-darwin"
            ;;
        mingw64_nt*)
            echo "unknown-linux-gnu"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

get_architecture() {
    case $(uname -m) in
        x86_64*)
            echo "x86_64"
            ;;
        aarch64*)
            echo "aarch64"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

environment_validation() {
    environment=$(get_platform)
    if [ $environment = "unknown" ]; then
        exit 1
    fi

    architecture=$(get_architecture)
    if [ $architecture = "unknown" ]; then
        exit 1
    fi

    if [ $(command -v wget >/dev/null 2>&1) ]; then
        exit 2
    fi

    echo "$environment:$architecture"
}

print_system_information() {
    echo "System environment: $(uname)"
    echo "CPU architecture: $(uname -m)"
}

remove_old_installation() {
    echo "Removing old installation..."
    rm -rf "$HOME/.tag-track"

    if [ -e "$HOME/.bashrc" ]; then
        sed -i "/$(echo "$PROFILE_MOD_EXPORT" | sed 's/[\/&]/\\&/g')/d" "$HOME/.bashrc"
        sed -i "/$(echo "$PROFILE_MOD_SOURCE" | sed 's/[\/&]/\\&/g')/d" "$HOME/.bashrc"
    fi

    if [ -e "$HOME/.zshrc" ]; then
        sed -i "/$(echo "$PROFILE_MOD_EXPORT" | sed 's/[\/&]/\\&/g')/d" "$HOME/.zshrc"
        sed -i "/$(echo "$PROFILE_MOD_SOURCE" | sed 's/[\/&]/\\&/g')/d" "$HOME/.zshrc"
    fi

    if [ -e "$HOME/.fishrc" ]; then
        sed -i "/$(echo "$PROFILE_MOD_EXPORT" | sed 's/[\/&]/\\&/g')/d" "$HOME/.fishrc"
        sed -i "/$(echo "$PROFILE_MOD_SOURCE" | sed 's/[\/&]/\\&/g')/d" "$HOME/.fishrc"
    fi
}

install_tag_track() {
    mkdir -p "$HOME/.tag-track/bin"
    binary="tag-track_${architecture}-${platform}"
    echo "Downloading version ${version}..."
    download_output=$(wget "${GITHUB_RELEASE_DOWNLOAD_URL}/${version}/${binary}" -O "$HOME/.tag-track/bin/tag-track" >/dev/null 2>&1)
    
    if [ $? -gt 0 ]; then
        exit $?
    fi

    echo "Installing tag-track at '$HOME/.tag-track/bin/tag-track'"
    chmod +x "$HOME/.tag-track/bin/tag-track"
    echo "$TAG_TRACK_SH" > "$HOME/.tag-track/tag-track.sh"

    if [ -e "$HOME/.bashrc" ]; then
        echo "$PROFILE_MOD_EXPORT" >> "$HOME/.bashrc"
        echo "$PROFILE_MOD_SOURCE" >> "$HOME/.bashrc"
    fi

    if [ -e "$HOME/.zshrc" ]; then
        echo "$PROFILE_MOD_EXPORT" >> "$HOME/.zshrc"
        echo "$PROFILE_MOD_SOURCE" >> "$HOME/.zshrc"
    fi

    if [ -e "$HOME/.fishrc" ]; then
        echo "$PROFILE_MOD_EXPORT" >> "$HOME/.fishrc"
        echo "$PROFILE_MOD_SOURCE" >> "$HOME/.fishrc"
    fi
}


GITHUB_REPOSITORY_BASE_URL="https://github.com/dloez"
GITHUB_RELEASES_BASE_URL="${GITHUB_REPOSITORY_BASE_URL}/tag-track/releases"
GITHUB_ISSUES_NEW_URL="${GITHUB_REPOSITORY_BASE_URL}/issues/new"
GITHUB_RELEASE_URL="${GITHUB_RELEASES_BASE_URL}/tag"
GITHUB_RELEASE_DOWNLOAD_URL="${GITHUB_RELEASES_BASE_URL}/download"

PROFILE_MOD_EXPORT='export TAG_TRACK_DIR="$HOME/.tag-track"'
PROFILE_MOD_SOURCE='[ -s "$HOME/.tag-track/tag-track.sh" ] && source "$HOME/.tag-track/tag-track.sh"'

TAG_TRACK_SH='export PATH=$PATH:$HOME/.tag-track/bin'

if [ $# -eq 0 ]; then
    echo "Missing version argument"
    exit 1
fi

version=$1
environment=$(environment_validation)
case $? in
    1)
        echo "The installation script is not able to determine your environment or \
is not currently supported. Please create an issue in ${GITHUB_ISSUES_NEW_URL} with the following information:"
        print_system_information
        exit 1
        ;;
    2)
        echo "The tool wget is required. Please check the 'Install method: Automatic script' - 'Requirements' section \
in ${GITHUB_RELEASE_URL}/${version}"
        exit 1
        ;;
esac

platform=$(echo $environment | cut -d : -f 1)
architecture=$(echo $environment | cut -d : -f 2)

if [ -e "$HOME/.tag-track" ]; then
    remove_old_installation
fi

install_tag_track
if [ $? -gt 0 ]; then
    echo "Failed to install tag-track"
    exit 1
fi

echo "Done! To start using tag-track close this shell and open a new one or run 'source "$HOME/.tag-track/tag-track.sh"'"
