use crate::{
    FixedSizeArrray,
    utils::{StringRef, StringRefExt as _, getenv},
};
use anyhow::{Context as _, Result, bail};
use libc::{_exit, O_WRONLY, STDERR_FILENO, STDOUT_FILENO, close, dup2, execvp, fork, open};

fn try_spawn(cmd: &str) -> Result<()> {
    let home =
        core::str::from_utf8(getenv(c"HOME").context("no $HOME")?).context("non-utf8 $HOME")?;

    let mut cmd = cmd.split_whitespace();
    let exe = StringRef::new(cmd.next().context("command can't be parsed")?);

    let mut argv: FixedSizeArrray<10, StringRef> =
        FixedSizeArrray::empty_with_default_fn(|| StringRef::new(""));
    argv.push(exe.clone())?;
    for arg in cmd {
        let arg = StringRef::new(&arg.replace('~', home));
        argv.push(arg)?;
    }

    let mut c_argv = FixedSizeArrray::<10, *mut i8>::new();
    for idx in 0..argv.len() {
        let arg = argv.get(idx).context("malformed state")?;
        c_argv.push(arg.as_const_ptr().cast_mut())?;
    }
    c_argv.push(core::ptr::null_mut())?;

    unsafe {
        let childpid = fork();

        if childpid < 0 {
            bail!("failed to fork")
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

pub(crate) fn spawn(cmd: &str) {
    if let Err(err) = try_spawn(cmd) {
        log::error!("{err:?}");
    }
}
