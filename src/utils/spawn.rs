use crate::utils::{StringRef, StringRefExt as _, getenv};
use alloc::vec::Vec;
use anyhow::{Context as _, Result, bail};
use libc::{_exit, O_WRONLY, STDERR_FILENO, STDOUT_FILENO, close, dup2, execvp, fork, open};

fn try_spawn(cmd: &str) -> Result<()> {
    let home =
        core::str::from_utf8(getenv(c"HOME").context("no $HOME")?).context("non-utf8 $HOME")?;

    let mut cmd = cmd.split_whitespace();
    let exe = StringRef::new(cmd.next().context("command can't be parsed")?);
    let argv = core::iter::once(exe.clone())
        .chain(cmd.map(|arg| StringRef::new(&arg.replace('~', home))))
        .collect::<Vec<_>>();
    let mut c_argv = argv
        .iter()
        .map(|arg| arg.as_const_ptr().cast_mut())
        .collect::<Vec<_>>();
    c_argv.push(core::ptr::null_mut());

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
