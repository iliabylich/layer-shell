fn main() {
    let path = "/usr/lib/x86_64-linux-gnu/girepository-1.0";
    println!("cargo::rustc-link-search=native={path}");
    println!("cargo::rustc-link-lib=gvc");
    println!("cargo::rustc-link-arg=-Wl,-rpath,{path}");
}
