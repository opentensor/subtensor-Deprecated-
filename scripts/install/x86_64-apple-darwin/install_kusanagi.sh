echo "******************************************************************"
echo " Installing subtensor as standalone node for the KUSANAGI network "
echo "******************************************************************"

echo "[+] Creating directory for chain data"
mkdir -p ~/Library/Application\ Support/node-subtensor

echo "[+] Setting up genesis block"
tar -xf ./kusanagi_genesis.tar -C ~/Library/Application\ Support/node-subtensor

echo "[+] Installation complete"
echo "[+] Run your node with the following command:"
echo "./node-subtensor"