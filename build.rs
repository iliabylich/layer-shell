fn main() {
    cc::Build::new().file("src/external.c").compile("external");

    println!("cargo::rustc-link-lib=uring");
}
