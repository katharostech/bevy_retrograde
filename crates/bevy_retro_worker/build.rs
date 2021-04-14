fn main() {
    cfg_aliases::cfg_aliases! {
        wasm: { target_arch = "wasm32" },
    }

    let web_worker_uri = std::env::var("BEVY_RETRO_WEB_WORKER_URI").unwrap_or("./worker.js".into());
    std::fs::write(
        &format!(
            "{}/{}",
            std::env::var("OUT_DIR").unwrap(),
            "web_worker_uri.txt"
        ),
        web_worker_uri,
    )
    .unwrap();
    println!("cargo:rerun-if-env-changed=BEVY_RETRO_WEB_WORKER_URI");
}
