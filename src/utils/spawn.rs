use crate::{
    FixedSizeArrray,
    utils::{ArrayWriter, StringRef, StringRefExt as _},
};
use core::fmt::Write;
use libc::{_exit, O_WRONLY, STDERR_FILENO, STDOUT_FILENO, close, dup2, execvp, fork, open};

fn try_spawn(cmd: &str, home: &str) -> Result<(), Error> {
    let mut cmd = cmd.split_whitespace();
    let exe = StringRef::new(cmd.next().ok_or(Error::EmptyCommand)?);

    let mut argv: FixedSizeArrray<10, StringRef> =
        FixedSizeArrray::empty_with_default_fn(|| StringRef::new(""));
    argv.push(exe.clone()).ok_or(Error::TooManyArguments)?;
    for arg in cmd {
        let arg = expand_home(arg, home);
        argv.push(arg).ok_or(Error::TooManyArguments)?;
    }

    let mut c_argv = FixedSizeArrray::<10, *mut i8>::new();
    for idx in 0..argv.len() {
        let arg = argv.get(idx).ok_or(Error::MalformedState { idx })?;
        c_argv
            .push(arg.as_const_ptr().cast_mut())
            .ok_or(Error::TooManyArguments)?;
    }
    c_argv
        .push(core::ptr::null_mut())
        .ok_or(Error::TooManyArguments)?;

    unsafe {
        let childpid = fork();

        if childpid < 0 {
            return Err(Error::Fork);
        }

        if childpid == 0 {
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
        write!(&mut writer, "{part}").unwrap_or_else(|_| unreachable!());
    }
    for part in parts {
        write!(&mut writer, "{home}{part}").unwrap_or_else(|_| unreachable!());
    }
    let s = core::str::from_utf8(writer.as_bytes()).unwrap_or_else(|_| {
        unreachable!("replacement of UTF-8 substrings can't make a string invalid")
    });
    StringRef::new(s)
}

pub(crate) fn spawn(cmd: &str, home: &str) {
    if let Err(err) = try_spawn(cmd, home) {
        log::error!("{err:?}");
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("command can't be parsed")]
    EmptyCommand,
    #[error("too many arguments")]
    TooManyArguments,
    #[error("malformed argv state at index {idx}")]
    MalformedState { idx: usize },
    #[error("failed to fork")]
    Fork,
}
