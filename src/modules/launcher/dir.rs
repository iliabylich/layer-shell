use crate::epoll::FdId;
use anyhow::{Context as _, Result};

pub(crate) trait WatcherDir {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn path(&self) -> String;
    fn fd_id(&self) -> FdId;
}

pub(crate) fn glob<T>(dir: &T) -> Result<Vec<String>>
where
    T: WatcherDir,
{
    let mut out = vec![];

    let path = dir.path();
    if let Ok(readdir) = std::fs::read_dir(&path) {
        for entry in readdir {
            let file = entry.with_context(|| format!("failed to get file list of {path}"))?;
            let path = file.path();
            if path.extension().is_some_and(|ext| ext == "desktop") {
                out.push(path.to_str().context("non-utf8 path")?.to_string());
            }
        }
    }

    Ok(out)
}

pub(crate) struct GlobalDir {
    path: String,
}

impl WatcherDir for GlobalDir {
    fn new() -> Result<Self> {
        let path = validate_exists(String::from("/usr/share/applications"))?;
        Ok(Self { path })
    }
    fn path(&self) -> String {
        self.path.clone()
    }

    fn fd_id(&self) -> FdId {
        FdId::LauncherGlobalDirInotify
    }
}

pub(crate) struct UserDir {
    path: String,
}

impl WatcherDir for UserDir {
    fn new() -> Result<Self> {
        let path = validate_exists(format!(
            "{}/.local/share/applications",
            std::env::var("HOME").context("no $HOME")?
        ))?;
        Ok(Self { path })
    }
    fn path(&self) -> String {
        self.path.clone()
    }

    fn fd_id(&self) -> FdId {
        FdId::LauncherUserDirInotify
    }
}

fn validate_exists(path: String) -> Result<String> {
    match std::fs::read_dir(&path) {
        Ok(_) => Ok(path),
        Err(err) => Err(anyhow::Error::from(err)
            .context(format!("Failed to access {path} because is not readable"))),
    }
}
