#!/bin/bash

UNIT_FILE="/etc/systemd/system/subtensor.service"
DATA_DIR="/var/lib/subtensor"
USERNAME="subtensor"
BINARY="node-subtensor"
CHAIN_DATA="$DATA_DIR/chains/kusanagi_mainnet/db"
CHAIN_TAR="./kusanagi_genesis.tar"
NETWORK="KUSANAGI"

abort() {
  printf "%s\n" "$1"
  exit 1
}

if [[ $EUID -ne 0 ]]; then
   abort "This script must be run as root"
fi

echo "*************************************************************************"
echo "This will install subtensor as a FULL node for the KUSANAGI network"
echo "*************************************************************************"
echo ""
echo "Warning! Any chain data present for this network will be deleted."
echo "Press a key to continue"
read


echo "[+] Copying ./$BINARY to /usr/local/bin/"
cp .$BINARY /usr/local/bin

id -u $USERNAME &>/dev/null || (echo "[+] Creating user subtensor" && useradd --no-create-home --shell /bin/false $USERNAME)
echo "[+] Creating data dir $DATA_DIR"
mkdir -p $DATA_DIR

echo "[+] Checking if $NETWORK chain data is already present"
if [ -d $CHAIN_DATA ] ; then
    echo "[!] $NETWORK chain data is already present, skipping initialization of genesis block."
else
    echo "[+] $NETWORK chain data is not present. This indicates a fresh install. Installing genesis block"
    tar -xf .$CHAIN_TAR -C $DATA_DIR
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
echo "Note: subtensor is currently NOT STARTED"
echo "Note: subtensor is currently NOT ENABLED"
echo ""
echo "--==[[ USEFUL COMMANDS ]]==--"
echo "Start subtensor : sudo systemctl start subtensor"
echo "Stop subtensor  : sudo systemctl stop subtensor"
echo "Start on reboot : sudo systemctl enable subtensor"
echo "Check status    : sudo systemctl status subtensor"

