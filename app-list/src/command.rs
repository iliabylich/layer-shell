#[derive(Debug)]
pub enum Command {
    Reset,
    GoUp,
    GoDown,
    SetSearch(String),
    ExecSelected,
}
