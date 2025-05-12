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

[INFO] Installing suiql-up, the version manager for SUIQL
EOF
}

remove_old_version() {
    echo "[INFO] Removing old version of suiql-up (if any)"
    sudo rm -f /usr/local/bin/suiql-up
    echo "[INFO] Old version removed (if existed)"
}

download_suiql_up() {
    echo "[INFO] Downloading latest suiql-up..."
    curl -fsSL -o suiql-up.sh "$SUIQL_UP_URL"
    chmod +x suiql-up.sh
}

move_suiql_up() {
    echo "[INFO] Moving suiql-up to /usr/local/bin..."
    sudo mv suiql-up.sh /usr/local/bin/suiql-up
}

final_message() {
    echo "---------------------- Installation complete ----------------------"
    echo ">>> Run 'suiql-up' to install or manage SUIQL versions"
}

main() {
    initial_message
    remove_old_version
    download_suiql_up
    move_suiql_up
    final_message
}

main
