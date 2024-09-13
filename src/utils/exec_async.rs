use gtk4::gio::{Subprocess, SubprocessFlags};
use std::ffi::OsStr;

pub(crate) async fn exec_async(cmd: &[&str]) -> String {
    let mut argv = vec![];
    for arg in cmd {
        argv.push(OsStr::new(*arg));
    }
    let process = Subprocess::newv(
        &argv,
        SubprocessFlags::STDOUT_PIPE | SubprocessFlags::STDERR_PIPE,
    )
    .unwrap();

    let (stdout, stderr) = process.communicate_utf8_future(None).await.unwrap();

    if process.is_successful() {
        stdout.unwrap_or_default().to_string()
    } else {
        stderr.unwrap_or_default().to_string()
    }
}
