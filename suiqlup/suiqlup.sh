#!/bin/bash

USERNAME="sand-worm-labs"
REPO_NAME="sandworm-sui-ql"
REPO_URL="https://github.com/${USERNAME}/${REPO_NAME}"
REPO_API_URL="https://api.github.com/repos/${USERNAME}/${REPO_NAME}"
CONFIG_FILE_URL="https://raw.githubusercontent.com/${USERNAME}/${REPO_NAME}/main/suiqlup/default-config.json"

LINUX_ASSET="suiqlup"
MAC_ASSET="suiqlup"

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

        ((( The suiqlup version manager )))
    "

    echo "[INFO] Installing the latest version of suiqlup: $LATEST_RELEASE_TAG"
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
        echo "${REPO_URL}/releases/latest"
        exit 1
    else
        echo "[INFO] Unsupported OS"
        exit 1
    fi
}

download_asset() {
    echo "[INFO] Downloading asset"
    curl -L -o suiqlup-release "${REPO_URL}/releases/download/${LATEST_RELEASE_TAG}/${ASSET_NAME}"
    echo "[INFO] Asset downloaded"
}

move_to_bin() {
    echo "[INFO] Moving to /usr/local/bin"
    sudo mv suiqlup-release /usr/local/bin/suiqlup
    chmod +x /usr/local/bin/suiqlup
    echo "[INFO] Installed to /usr/local/bin/suiqlup"
}

remove_old_version() {
    echo "[INFO] Removing old version of suiqlup"
    sudo rm -f /usr/local/bin/suiqlup
    echo "[INFO] Old version removed"
}

clone_chains_file_if_needed() {
    if [ ! -f ~/suiql-config.json ]; then
        echo "[INFO] Cloning default SUIQL config file to ~/suiql-config.json"
        curl -L -s -o suiql-config.json $CONFIG_FILE_URL
        mv suiql-config.json ~/suiql-config.json
    else
        echo "[INFO] SUIQL config file already exists. Skipping"
    fi
}

cleanup() {
    rm -rf latest latest.zip
    echo "[INFO] Cleaned up"
}

final_message() {
    echo "---------------------- Installation complete ----------------------"
    echo ">>> Run 'suiqlup --help' to get started"
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
