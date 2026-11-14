// The web pages are baked into the program from `../web/dist`. That folder
// only exists after `npm run build`. Make an empty one if it is missing, so
// `cargo test` works on its own.
fn main() {
    let _ = std::fs::create_dir_all("../web/dist");
    println!("cargo:rerun-if-changed=../web/dist");
}
