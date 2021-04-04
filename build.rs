fn main() {
    cfg_aliases::cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        webgl1: { all(wasm, feature = "webgl1") },
    }
}
