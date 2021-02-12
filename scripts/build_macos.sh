rustup target add x86_64-apple-darwin
cargo build --release --target=x86_64-apple-darwin
cp ./target/release/node-subtensor ./bin/macos_x86_64