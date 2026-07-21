fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let external = std::path::Path::new(&manifest_dir).join("src/external.c");
    let external = if external.exists() {
        external
    } else {
        std::path::Path::new(&manifest_dir).join("../src/external.c")
    };

    cc::Build::new().file(external).compile("external");

    println!("cargo::rustc-link-lib=uring");
}
