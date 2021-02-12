#!/bin/bash

abort() {
  printf "%s\n" "$1"
  exit 1
}

if [[ $EUID -ne 0 ]]; then
   abort "This script must be run as root"
fi

NODE_TYPE=$1
NETWORK=$2
DATA_DIR="/var/lib/subtensor"


if [ "$NODE_TYPE" = "LIGHT" ] ; then
    LIGHT_NODE_FLAG="--light"
    NODE_TYPE_LOWERCASE="light"
elif [ "$NODE_TYPE" = "FULL" ]; then
    LIGHT_NODE_FLAG=""
    NODE_TYPE_LOWERCASE="full"
else
    echo "[!] Unknown node type ($NODE_TYPE). Aborting"
    exit
fi

if [ "$NETWORK" = "KUSANAGI" ] ; then
  NETWORK_LOWERCASE='kusanagi'
  CHAIN_DATA="$DATA_DIR/chains/kusanagi_mainnet/db"
  AKIRA_CHAIN_FLAG=""
elif [ "$NETWORK" = "AKIRA" ] ; then
  NETWORK_LOWERCASE='akira'
  CHAIN_DATA="$DATA_DIR/chains/akira_testnet/db"
  AKIRA_CHAIN_FLAG="--chain akira"
else
  echo "Unkown network ($NETWORK) specified. Aborting"
  exit
fi

CHAIN_TAR="./bin/release/$NETWORK_LOWERCASE""_genesis_$NODE_TYPE_LOWERCASE.tar"


echo "*************************************************************************"
echo "This will install subtensor as a $NODE_TYPE node for the $NETWORK network"
echo "*************************************************************************"
echo ""
echo "Warning! Any chain data present for this network will be deleted."
echo "Press a key to continue"
read


UNIT_FILE="/etc/systemd/system/subtensor.service"

USERNAME="subtensor"
echo "[+] Copying ./bin/release/node-subtensor to /usr/local/bin/"
# Switch binary based on distro.
OS="$(uname)"
if [[ "$OS" == "Linux" ]]; then
  BINARY="./bin/linux_x86_64/node-subtensor"
elif [[ "$OS" == "Darwin" ]]; then
  BINARY="./bin/macos_x86_64/node-subtensor"
else
  abort "There is no compatible subtensor binary for you operating system. You need to build from source."
fi
cp $BINARY /usr/local/bin

id -u $USERNAME &>/dev/null || (echo "[+] Creating user subtensor" && useradd --no-create-home --shell /bin/false $USERNAME)
echo "[+] Creating data dir $DATA_DIR"
mkdir -p $DATA_DIR

echo "[+] Checking if $NETWORK_LOWERCASE chain data is already present"
if [ -d $CHAIN_DATA ] ; then
    echo "[!] $NETWORK_LOWERCASE chain data is already present, deleting .."
    rm -rf $CHAIN_DATA
fi

echo "[+] Installing genesis block"
tar -xf $CHAIN_TAR -C $DATA_DIR


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
ExecStart=/usr/local/bin/node-subtensor --base-path $DATA_DIR $LIGHT_NODE_FLAG $AKIRA_CHAIN_FLAG
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