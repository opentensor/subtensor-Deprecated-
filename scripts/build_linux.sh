rustup target add x86_64-unknown-linux-gnu
cargo build --release
echo "[+] Copying binary from ./target/release/node-subtensor to ./bin/linux_x86_64/node-subtensor"
cp ./target/release/node-subtensor ./bin/linux_x86_64
