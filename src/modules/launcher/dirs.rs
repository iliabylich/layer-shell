use crate::macros::fatal;
use anyhow::{Context as _, Result};

pub(crate) fn glob(dir: &str) -> Result<Vec<String>> {
    let mut out = vec![];

    if let Ok(readdir) = std::fs::read_dir(dir) {
        for entry in readdir {
            let file = entry.with_context(|| format!("failed to get file list of {dir}"))?;
            let path = file.path();
            if path.extension().is_some_and(|ext| ext == "desktop") {
                out.push(path.to_str().context("non-utf8 path")?.to_string());
            }
        }
    }

    Ok(out)
}

pub(crate) fn global_dir() -> Option<String> {
    validate_exists(String::from("/usr/share/applications"))
}

pub(crate) fn user_dir() -> Option<String> {
    validate_exists(format!(
        "{}/.local/share/applications",
        std::env::var("HOME").unwrap_or_else(|_| fatal!("no $HOME"))
    ))
}

fn validate_exists(path: String) -> Option<String> {
    match std::fs::read_dir(&path) {
        Ok(_) => Some(path),
        Err(err) => {
            log::warn!("Skipping dir {path} because is not readable: {:?}", err);
            None
        }
    }
}
