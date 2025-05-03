USERNAME="iankressin"
REPO_NAME="sui-ql"
EQLUP_URL="https://raw.githubusercontent.com/${USERNAME}/${REPO_NAME}/main/sui-qlup/sui-qlup.sh"

initial_message() {
    echo "

  ██████  █    ██  ██▓  █████   ██▓    
▒██    ▒  ██  ▓██▒▓██▒▒██▓  ██▒▓██▒    
░ ▓██▄   ▓██  ▒██░▒██▒▒██▒  ██░▒██░    
  ▒   ██▒▓▓█  ░██░░██░░██  █▀ ░▒██░    
▒██████▒▒▒▒█████▓ ░██░░▒███▒█▄ ░██████▒
▒ ▒▓▒ ▒ ░░▒▓▒ ▒ ▒ ░▓  ░░ ▒▒░ ▒ ░ ▒░▓  ░
░ ░▒  ░ ░░░▒░ ░ ░  ▒ ░ ░ ▒░  ░ ░ ░ ▒  ░
░  ░  ░   ░░░ ░ ░  ▒ ░   ░   ░   ░ ░   
      ░     ░      ░      ░        ░  ░
 
    "

    echo "[INFO] Installing sui-qlup, the version manager of EQL"
}

remove_old_version() {
    echo "[INFO] Removing old version of sui-qlup"
    sudo rm -f /usr/local/bin/sui-qlup
    echo "[INFO] Old version removed "
}

download_sui-qlup() {
    curl -s -o sui-qlup.sh $EQLUP_URL
    chmod +x sui-qlup.sh
}

move_sui-qlup() {
    sudo mv sui-qlup.sh /usr/local/bin/sui-qlup
}

final_message() {
    echo "---------------------- Installation complete ----------------------"
    echo ">>> Run 'sui-qlup' to install EVM Query Language (EQL)"
}

main() {
    initial_message
    remove_old_version
    download_sui-qlup
    move_sui-qlup
    final_message
}

main
