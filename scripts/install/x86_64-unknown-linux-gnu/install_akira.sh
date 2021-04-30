echo "***************************************************************"
echo " Installing subtensor as standalone node for the AKIRA network "
echo "***************************************************************"

echo "[+] Creating directory for chain data"
mkdir -p ~/.local/share/node-subtensor

echo "[+] Setting up genesis block"
tar -xf ./akira_genesis.tar -C ~/.local/share/node-subtensor

echo "[+] Installation complete"
echo "[+] Run your node with the following command:"
echo "./node-subtensor --chain akira"
