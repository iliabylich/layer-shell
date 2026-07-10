fn main() {
    cc::Build::new().file("src/external.c").compile("external");

    println!("cargo::rustc-link-lib=uring");
    println!("cargo::rustc-link-lib=ssl");
    println!("cargo::rustc-link-lib=crypto");
}
