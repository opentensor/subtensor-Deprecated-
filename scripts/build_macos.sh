rustup target add x86_64-apple-darwin
cargo build --release --target=x86_64-apple-darwin
echo "[+] Copying binary from ./target/release/node-subtensor to ./bin/macos_x86_64/node-subtensor"
cp ./target/release/node-subtensor ./bin/macos_x86_64