# This is a justfile. See https://github.com/casey/just

list:
    just --list

check:
    cargo clippy -- -W clippy::correctness -D warnings
    cargo fmt --check

build:
    cargo build

build-release:
    cargo build --release

build-web $CARGO_TARGET_DIR='web-target':
    cargo build --target wasm32-unknown-unknown
    wasm-bindgen --out-dir $CARGO_TARGET_DIR/web-dist --target web $CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/punchy.wasm
    cat wasm_resources/index.html | sed "s/\$BASEPATH//g" > $CARGO_TARGET_DIR/web-dist/index.html
    mkdir -p target/web-dist
    cp -r assets $CARGO_TARGET_DIR/web-dist/

build-release-web basepath='' $CARGO_TARGET_DIR='web-target':
    cargo build --target wasm32-unknown-unknown --release
    wasm-bindgen --out-dir $CARGO_TARGET_DIR/web-dist --no-typescript --target web $CARGO_TARGET_DIR/wasm32-unknown-unknown/release/punchy.wasm
    cat wasm_resources/index.html | sed "s/\$BASEPATH/$(printf {{basepath}} | sed 's/\//\\\//g')/g" > $CARGO_TARGET_DIR/web-dist/index.html
    cp -r assets $CARGO_TARGET_DIR/web-dist/

run *args:
    cargo run -- {{args}}

run-web port='4000' host='127.0.0.1': build-web
    @echo "Debug link: http://{{host}}:{{port}}?RUST_LOG=debug"
    basic-http-server -a '{{host}}:{{port}}' -x web-target/web-dist