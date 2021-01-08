#!/bin/bash

if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root"
   exit 1
fi



UNIT_FILE="/etc/systemd/system/subtensor.service"
DATA_DIR="/var/lib/subtensor"
USERNAME="subtensor"
BINARY="node-subtensor"


echo "[+] Copying ./bin/release/node-subtensor to /usr/local/bin/"
cp ./bin/release/$BINARY /usr/local/bin

id -u $USERNAME &>/dev/null || (echo "[+] Creating user subtensor" && useradd --no-create-home --shell /bin/false $USERNAME)
echo "[+] Creating data dir $DATA_DIR"
mkdir -p $DATA_DIR

echo "[+] Setting ownership of $DATA_DIR to $USERNAME:$USERNAME"
chown $USERNAME:$USERNAME $DATA_DIR

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