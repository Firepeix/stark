help:
    @just --list

run:
    cargo build; cargo run -- 8000;    

run-release-local $REMOTE_CONFIG_URL="http://192.168.0.11:3000" $AUTHENTICATION_URL="http://192.168.0.11:3000" $NGROK_PATH="./ngrok":
    just run

run-release $REMOTE_CONFIG_URL="https://firebaseremoteconfig.googleapis.com" $AUTHENTICATION_URL="https://oauth2.googleapis.com" $NGROK_PATH="./ngrok":
    just run

release:
    cp -r ./src ../RustCompiler/package/
    cp -r ./Cargo.lock ../RustCompiler/package/
    cp -r ./Cargo.toml ../RustCompiler/package/
    docker exec rustc sh -c "cd app && cargo build --release"
    cp ../RustCompiler/package/target/release/stark bin/stark 
    sudo rm -rf ../RustCompiler/package/*

