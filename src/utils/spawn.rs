use crate::{
    FixedSizeArrray,
    utils::{ArrayWriter, StringRef, StringRefExt as _, log_err_and_exit},
};
use core::fmt::Write;
use libc::{_exit, O_WRONLY, STDERR_FILENO, STDOUT_FILENO, close, dup2, execvp, fork, open};

pub struct SpawnHelper;

impl SpawnHelper {
    pub(crate) fn spawn(cmd: &str, home: &str) {
        log::trace!("Spawning {cmd:?}");

        let _ = try_spawn(cmd, home);
    }
}

fn try_spawn(cmd: &str, home: &str) -> Result<(), ()> {
    let mut cmd = cmd.split_whitespace();
    let exe = StringRef::new(cmd.next().ok_or_else(|| {
        log::error!("empty command");
    })?);

    let mut argv = FixedSizeArrray::<10, _>::empty_with_default_fn(StringRef::empty);
    argv.push(exe.clone()).ok_or_else(|| {
        log::error!("command is too long");
    })?;
    for arg in cmd {
        let arg = expand_home(arg, home);
        argv.push(arg).ok_or_else(|| {
            log::error!("command is too long");
        })?;
    }

    let mut c_argv = FixedSizeArrray::<10, *mut i8>::new();
    for idx in 0..argv.len() {
        let Some(arg) = argv.get(idx) else {
            log::error!("malformed state: failed to get index {idx}");
            return Err(());
        };
        c_argv.push(arg.as_const_ptr().cast_mut()).ok_or_else(|| {
            log::error!("command is too long");
        })?;
    }
    c_argv.push(core::ptr::null_mut()).ok_or_else(|| {
        log::error!("command is too long");
    })?;

    unsafe {
        let res = fork();

        if res < 0 {
            log::error!("failed to fork: {res}");
            return Err(());
        }

        if res == 0 {
            let dev_null = c"/dev/null";
            let fd = open(dev_null.as_ptr(), O_WRONLY);
            if fd >= 0 {
                dup2(fd, STDOUT_FILENO);
                dup2(fd, STDERR_FILENO);

                if fd != STDOUT_FILENO && fd != STDERR_FILENO {
                    close(fd);
                }
            }
            execvp(exe.as_const_ptr(), c_argv.as_ptr().cast());
            _exit(127);
        }

        Ok(())
    }
}

fn expand_home(arg: &str, home: &str) -> StringRef {
    if !arg.as_bytes().contains(&b'~') {
        return StringRef::new(arg);
    }

    let mut buf = [0; 256];
    let mut writer = ArrayWriter::new(&mut buf);
    let mut parts = arg.split('~');
    if let Some(part) = parts.next() {
        write!(&mut writer, "{part}")
            .unwrap_or_else(|_| log_err_and_exit!("command is too long for 256 bytes long buffer"));
    }
    for part in parts {
        write!(&mut writer, "{home}{part}")
            .unwrap_or_else(|_| log_err_and_exit!("command is too long for 256 bytes long buffer"));
    }
    let s = core::str::from_utf8(writer.as_bytes()).unwrap_or_else(|_| {
        log_err_and_exit!("replacement of UTF-8 substrings can't make a string invalid")
    });
    StringRef::new(s)
}
