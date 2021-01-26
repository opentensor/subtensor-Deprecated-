#!/bin/bash

if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root"
   exit 1
fi



UNIT_FILE="/etc/systemd/system/subtensor.service"
DATA_DIR="/var/lib/subtensor"
USERNAME="subtensor"
BINARY="node-subtensor"
CHAIN_DATA="$DATA_DIR/chains/kusanagi_mainnet/db"

echo "[+] Copying ./bin/release/node-subtensor to /usr/local/bin/"
cp ./bin/release/$BINARY /usr/local/bin

id -u $USERNAME &>/dev/null || (echo "[+] Creating user subtensor" && useradd --no-create-home --shell /bin/false $USERNAME)
echo "[+] Creating data dir $DATA_DIR"
mkdir -p $DATA_DIR

echo "[+] Checking if kusanagi chain data is already present"
if [ -d $CHAIN_DATA ] ; then
    echo "[!] kusanagi chain data is already present, skipping initialization of genesis block."
else
    echo "[+] kusanagi chain data is not present. This indicates a fresh install. Installing genesis block"
    tar -xf ./bin/release/kusanagi_genesis.tar -C $DATA_DIR
fi

echo "[+] Setting ownership of $DATA_DIR and subdirs to $USERNAME:$USERNAME"
chown -R $USERNAME:$USERNAME $DATA_DIR

echo "[+] Creating unit file $UNIT_FILE"

cat << EOF > $UNIT_FILE
[Unit]
Description=Subtensor node

Wants=network.target
After=syslog.target network-online.target

[Service]
User=$USERNAME
Type=simple
ExecStart=/usr/local/bin/$BINARY --base-path $DATA_DIR
Restart=on-failure
RestartSec=10
KillMode=process

[Install]
WantedBy=multi-user.target
EOF

echo "[+] Done!"
echo ""
echo "--==[[ USEFUL COMMANDS ]]==--"
echo "Start subtensor : sudo systemctl start subtensor"
echo "Stop subtensor  : sudo systemctl stop subtensor"
echo "Start on reboot : sudo systemctl enable subtensor"
echo "Check status    : sudo systemctl status subtensor"