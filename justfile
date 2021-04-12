dev_features:="ldtk"
native_dev_features:="bevy/dynamic"

build:
    cargo build --features {{dev_features}},{{native_dev_features}}

run-example example='hello_world':
    cargo run --example {{example}} --features {{dev_features}},{{native_dev_features}}

run-example-web example='hello_world':
    cargo build --example {{example}} --features {{dev_features}} --target wasm32-unknown-unknown
    wasm-bindgen --out-dir target/wasm/{{example}} --target no-modules --no-modules-global {{example}} target/wasm32-unknown-unknown/debug/examples/{{example}}.wasm
    cp wasm_resources/index.tpl.html target/wasm/{{example}}/index.html
    sed -i s/\$example/{{example}}/g target/wasm/{{example}}/index.html
    ln -fs ../../../assets target/wasm/{{example}}
    basic-http-server target/wasm/{{example}}

doc *args:
    cargo doc --features {{dev_features}} {{args}}

readme:
    cargo readme > README.md
