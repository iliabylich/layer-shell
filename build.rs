fn main() {
    cc::Build::new()
        .file("src/liburing-wrapper.c")
        .compile("uring-wrapper");

    println!("cargo::rustc-link-lib=uring")
}
