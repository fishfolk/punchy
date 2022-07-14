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

build-web:
    cargo build --target wasm32-unknown-unknown
    wasm-bindgen --out-dir target/wasm --target web target/wasm32-unknown-unknown/debug/punchy.wasm
    cat wasm_resources/index.html | sed "s/\$BASEPATH//g" > target/wasm/index.html
    mkdir -p target/wasm
    cp -r assets target/wasm/

build-release-web basepath='':
    cargo build --target wasm32-unknown-unknown --release
    wasm-bindgen --out-dir target/wasm-dist --no-typescript --target web target/wasm32-unknown-unknown/release/punchy.wasm
    cat wasm_resources/index.html | sed "s/\$BASEPATH/$(printf {{basepath}} | sed 's/\//\\\//g')/g" > target/wasm-dist/index.html
    cp -r assets target/wasm-dist/

run *args:
    cargo run -- {{args}}

run-web port='4000' host='127.0.0.1': build-web
    @echo "Debug link: http://{{host}}:{{port}}?RUST_LOG=debug"
    basic-http-server -a '{{host}}:{{port}}' -x target/wasm