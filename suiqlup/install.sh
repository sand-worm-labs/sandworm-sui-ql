#!/bin/bash

USERNAME="sand-worm-labs"
REPO_NAME="sandworm-sui-ql"
SUIQL_UP_URL="https://raw.githubusercontent.com/${USERNAME}/${REPO_NAME}/main/suiql_up/suiql_up.sh"

initial_message() {
    cat << "EOF"

  ██████  █    ██  ██▓  █████   ██▓    
▒██    ▒  ██  ▓██▒▓██▒▒██▓  ██▒▓██▒    
░ ▓██▄   ▓██  ▒██░▒██▒▒██▒  ██░▒██░    
  ▒   ██▒▓▓█  ░██░░██░░██  █▀ ░▒██░    
▒██████▒▒▒▒█████▓ ░██░░▒███▒█▄ ░██████▒
▒ ▒▓▒ ▒ ░░▒▓▒ ▒ ▒ ░▓  ░░ ▒▒░ ▒ ░ ▒░▓  ░
░ ░▒  ░ ░░░▒░ ░ ░  ▒ ░ ░ ▒░  ░ ░ ░ ▒  ░
░  ░  ░   ░░░ ░ ░  ▒ ░   ░   ░   ░ ░   
      ░     ░      ░      ░        ░  ░

[INFO] Installing suiqlup, the version manager for SUIQL
EOF
}

remove_old_version() {
    echo "[INFO] Removing old version of suiqlup (if any)"
    sudo rm -f /usr/local/bin/suiqlup
    echo "[INFO] Old version removed (if existed)"
}

download_suiql_up() {
    echo "[INFO] Downloading latest suiqlup..."
    curl -fsSL -o suiqlup.sh "$SUIQL_UP_URL"
    chmod +x suiqlup.sh
}

move_suiql_up() {
    echo "[INFO] Moving suiqlup to /usr/local/bin..."
    sudo mv suiqlup.sh /usr/local/bin/suiqlup
}

final_message() {
    echo "---------------------- Installation complete ----------------------"
    echo ">>> Run 'suiqlup' to install or manage SUIQL versions"
}

main() {
    initial_message
    remove_old_version
    download_suiql_up
    move_suiql_up
    final_message
}

main
