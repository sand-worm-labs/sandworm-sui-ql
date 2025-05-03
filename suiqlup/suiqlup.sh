#!/bin/bash

USERNAME="iankressin"
REPO_NAME="sui-ql"
REPO_URL="https://github.com/{$USERNAME}/${REPO_NAME}"
REPO_API_URL="https://api.github.com/repos/${USERNAME}/${REPO_NAME}"
CONFIG_FILE_URL="https://raw.githubusercontent.com/${USERNAME}/${REPO_NAME}/main/sui-qlup/default-config.json"

LINUX_ASSET="sui-ql"
MAC_ASSET="sui-ql"

get_latest_release_tag() {
    LATEST_RELEASE_TAG=$(curl -s "${REPO_API_URL}/releases/latest" | sed -n 's/.*"tag_name": "\(.*\)".*/\1/p')
}

initial_message() {
    echo "

  ██████  █    ██  ██▓  █████   ██▓     █    ██  ██▓███  
▒██    ▒  ██  ▓██▒▓██▒▒██▓  ██▒▓██▒     ██  ▓██▒▓██░  ██▒
░ ▓██▄   ▓██  ▒██░▒██▒▒██▒  ██░▒██░    ▓██  ▒██░▓██░ ██▓▒
  ▒   ██▒▓▓█  ░██░░██░░██  █▀ ░▒██░    ▓▓█  ░██░▒██▄█▓▒ ▒
▒██████▒▒▒▒█████▓ ░██░░▒███▒█▄ ░██████▒▒▒█████▓ ▒██▒ ░  ░
▒ ▒▓▒ ▒ ░░▒▓▒ ▒ ▒ ░▓  ░░ ▒▒░ ▒ ░ ▒░▓  ░░▒▓▒ ▒ ▒ ▒▓▒░ ░  ░
░ ░▒  ░ ░░░▒░ ░ ░  ▒ ░ ░ ▒░  ░ ░ ░ ▒  ░░░▒░ ░ ░ ░▒ ░     
░  ░  ░   ░░░ ░ ░  ▒ ░   ░   ░   ░ ░    ░░░ ░ ░ ░░       
      ░     ░      ░      ░        ░  ░   ░              

        ((( The sui-ql version manager )))
    "

    echo "[INFO] Installing the lastest version of sui-ql: $LATEST_RELEASE_TAG"
}

detect_os() {
    if [[ "$OSTYPE" == "linux-gnu" ]]; then
        ASSET_NAME=$LINUX_ASSET
        echo "[INFO] Linux detected"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        ASSET_NAME=$MAC_ASSET
        echo "[INFO] MacOS detected"
    elif [[ "$OSTYPE" == "cygwin" ]]; then
        echo "[INFO] On Windows, download the executable from the link below:"
        echo "{ $REPO_URL }/releases/latest"
        exit 1
    else
        echo "[INFO] Unsupported OS"
        exit 1
    fi
}

download_asset() {
    echo "[INFO] Downloading asset"
    curl -L -o sui-ql-release "${REPO_URL}/releases/download/${LATEST_RELEASE_TAG}/${ASSET_NAME}"
    echo "[INFO] Asset downloaded"
}

move_to_bin() {
    echo "[INFO] Moving to /usr/local/bin"
    sudo mv sui-ql-release /usr/local/bin/sui-ql
    chmod +x /usr/local/bin/sui-ql
    echo "[INFO] Installed to /usr/local/bin/sui-ql"
}

remove_old_version() {
    echo "[INFO] Removing old version of sui-ql"
    sudo rm -f /usr/local/bin/sui-ql
    echo "[INFO] Old version removed "
}

clone_chains_file_if_needed() {
    if [ ! -f ~/sui-ql-config.json ]; then
        echo "[INFO] Cloning default EQL config file to ~/sui-ql-config.json"
        curl -L -s -o sui-ql-config.json $CONFIG_FILE_URL
        mv sui-ql-config.json ~/sui-ql-config.json
    else
        echo "[INFO] EQL config file already exists. Skipping"
    fi
}

cleanup() {
    rm -rf latest latest.zip
    echo "[INFO] Cleaned up"
}

final_message() {
    echo "---------------------- Installation complete ----------------------"
    echo ">>> Run 'sui-ql --help' to get started"
}

main() {
    get_latest_release_tag
    initial_message
    remove_old_version
    detect_os
    download_asset
    move_to_bin
    clone_chains_file_if_needed
    cleanup
    final_message
}

main
