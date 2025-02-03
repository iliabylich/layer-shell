use crate::macros::fatal;
use anyhow::{Context as _, Result};

pub(crate) fn dirlist() -> Vec<String> {
    let mut out = vec![];

    for dir in [global_dir(), user_dir()] {
        match std::fs::read_dir(&dir) {
            Ok(_) => out.push(dir),
            Err(err) => log::warn!("Skipping dir {dir} because is not readable: {:?}", err),
        }
    }
    out
}

pub(crate) fn filelist(dirs: &[String]) -> Result<Vec<String>> {
    let mut out = vec![];

    for dir in dirs {
        if let Ok(readdir) = std::fs::read_dir(dir) {
            for entry in readdir {
                let file = entry.with_context(|| format!("failed to get file list of {dir}"))?;
                let path = file.path();
                if path.extension().is_some_and(|ext| ext == "desktop") {
                    out.push(path.to_str().context("non-utf8 path")?.to_string());
                }
            }
        }
    }

    Ok(out)
}

fn global_dir() -> String {
    String::from("/usr/share/applications")
}

fn user_dir() -> String {
    format!(
        "{}/.local/share/applications",
        std::env::var("HOME").unwrap_or_else(|_| fatal!("no $HOME"))
    )
}
