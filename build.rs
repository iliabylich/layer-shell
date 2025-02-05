use std::{fs::File, time::SystemTime};

fn main() {
    let file = File::open("main.cpp").unwrap();
    file.set_modified(SystemTime::now()).unwrap();
}
