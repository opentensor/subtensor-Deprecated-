rustup target add x86_64-unknown-linux-gnu
cargo build --release
cp ./target/release/node-subtensor ./bin/linux_x86_64
