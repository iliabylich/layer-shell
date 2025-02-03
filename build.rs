use std::{fs::File, time::SystemTime};

fn main() {
    let file = File::open("main.c").unwrap();
    file.set_modified(SystemTime::now()).unwrap();
}
