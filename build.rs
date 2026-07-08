fn main() {
    cc::Build::new()
        .file("src/liburing-wrapper.c")
        .file("src/openssl-wrapper.c")
        .compile("uring-wrapper");

    println!("cargo::rustc-link-lib=uring");
    println!("cargo::rustc-link-lib=ssl");
    println!("cargo::rustc-link-lib=crypto");
}
