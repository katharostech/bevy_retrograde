development_features:="--features bevy/dynamic"

build:
    cargo build {{development_features}}

run-example example='hello_world':
    cargo run --example {{example}} {{development_features}}

run-example-web example='hello_world':
    cargo build --example {{example}} --target wasm32-unknown-unknown
    wasm-bindgen --out-dir target/wasm/{{example}} --target web target/wasm32-unknown-unknown/debug/examples/{{example}}.wasm
    cp wasm_resources/index.tpl.html target/wasm/{{example}}/index.html
    sed -i s/\$example/{{example}}/ target/wasm/{{example}}/index.html
    ln -fs ../../../assets target/wasm/{{example}}
    basic-http-server target/wasm/{{example}}

readme:
    cargo readme > README.md
