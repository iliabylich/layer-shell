#[derive(Debug, Clone)]
pub enum AppIcon {
    IconPath(String),
    IconName(String),
}

impl From<String> for AppIcon {
    fn from(s: String) -> Self {
        if s.starts_with('/') {
            AppIcon::IconPath(s)
        } else {
            AppIcon::IconName(s)
        }
    }
}
