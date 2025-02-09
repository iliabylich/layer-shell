#[derive(Debug)]
pub(crate) enum TrayCommand {
    Added { service: String, path: String },
    Removed { service: String },
    Updated { service: String },
}
