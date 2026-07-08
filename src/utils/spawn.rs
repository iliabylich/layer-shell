use crate::utils::{StringRef, StringRefExt as _, getenv};
use anyhow::{Context as _, Result, bail};

pub(crate) fn spawn(cmd: &str) -> Result<()> {
    let home =
        core::str::from_utf8(getenv(c"HOME").context("no $HOME")?).context("non-utf8 $HOME")?;

    let mut cmd = cmd.split_whitespace();
    let exe = StringRef::new(cmd.next().context("command can't be parsed")?);
    let argv = core::iter::once(exe.clone())
        .chain(cmd.map(|arg| StringRef::new(&arg.replace('~', home))))
        .collect::<Vec<_>>();
    let mut c_argv = argv.iter().map(StringRef::as_const_ptr).collect::<Vec<_>>();
    c_argv.push(core::ptr::null());

    unsafe {
        let childpid = libc::fork();

        if childpid < 0 {
            bail!("failed to fork")
        }

        if childpid == 0 {
            let dev_null = c"/dev/null";
            let fd = libc::open(dev_null.as_ptr(), libc::O_WRONLY);
            if fd >= 0 {
                libc::dup2(fd, libc::STDOUT_FILENO);
                libc::dup2(fd, libc::STDERR_FILENO);

                if fd != libc::STDOUT_FILENO && fd != libc::STDERR_FILENO {
                    libc::close(fd);
                }
            }
            libc::execvp(exe.as_const_ptr(), c_argv.as_ptr());
            libc::_exit(127);
        }

        Ok(())
    }
}
