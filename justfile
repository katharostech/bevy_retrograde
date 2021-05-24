dev_features:="ldtk"
native_dev_features:="bevy/dynamic"

# List the justfile recipes
list:
    just --list

# Generate the README from the lib.rs docs
readme:
    cargo doc2readme --template README.j2 --out README.md

# Build Bevy Retro
build:
    cargo build --features {{dev_features}},{{native_dev_features}}

# Run an example
run-example example='hello_world':
    cargo run --example {{example}} --features {{dev_features}},{{native_dev_features}}

# Build an example for web and host it on a local webserver
run-example-web example='hello_world':
    cargo build --example {{example}} --features {{dev_features}} --target wasm32-unknown-unknown
    wasm-bindgen --out-dir target/wasm/{{example}} --target web target/wasm32-unknown-unknown/debug/examples/{{example}}.wasm
    cp wasm_resources/index.tpl.html target/wasm/{{example}}/index.html
    sed -i s/\$example/{{example}}/ target/wasm/{{example}}/index.html
    ln -fs ../../../assets target/wasm/{{example}}
    basic-http-server target/wasm/{{example}}

# build the documentation
doc *args:
    cargo doc --features {{dev_features}} {{args}}
