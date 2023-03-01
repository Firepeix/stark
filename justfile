help:
    @just --list

run:
    cargo build; cargo run;    

run-release-local $REMOTE_CONFIG_URL="http://localhost:3000" $AUTHENTICATION_URL="http://localhost:3000":
    just run

run-release $REMOTE_CONFIG_URL="https://firebaseremoteconfig.googleapis.com" $AUTHENTICATION_URL="https://oauth2.googleapis.com":
    just run