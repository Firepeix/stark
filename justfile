help:
    @just --list

run:
    cargo build; cargo run -- http 8000;    

run-release-local $REMOTE_CONFIG_URL="http://192.168.0.11:3000" $AUTHENTICATION_URL="http://192.168.0.11:3000" $NGROK_PATH="./ngrok":
    just run

run-release $REMOTE_CONFIG_URL="https://firebaseremoteconfig.googleapis.com" $AUTHENTICATION_URL="https://oauth2.googleapis.com" $NGROK_PATH="./ngrok":
    just run

release:
    cargo build --release --target x86_64-unknown-linux-musl
    cp target/release/stark bin/stark  

