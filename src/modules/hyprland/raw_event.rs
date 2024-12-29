#[derive(Clone, Debug)]
pub(crate) enum RawEvent {
    CreateWorkspace(usize),
    DestroyWorkspace(usize),
    Workspace(usize),
    LanguageChanged(String),
}

impl RawEvent {
    pub(crate) fn parse(line: String) -> Option<Self> {
        log::info!("parsing {line:?}");

        let (event, payload) = line.split_once(">>")?;

        let payload_as_usize = || payload.parse::<usize>().ok();

        match event {
            "createworkspace" => Some(Self::CreateWorkspace(payload_as_usize()?)),
            "destroyworkspace" => Some(Self::DestroyWorkspace(payload_as_usize()?)),
            "workspace" => Some(Self::Workspace(payload_as_usize()?)),
            "activelayout" => {
                let lang = payload.split(",").last()?;
                Some(Self::LanguageChanged(lang.to_string()))
            }
            _ => None,
        }
    }
}
