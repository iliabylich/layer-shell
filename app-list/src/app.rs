use crate::AppIcon;

#[derive(Debug)]
pub struct App {
    pub name: String,
    pub selected: bool,
    pub icon: AppIcon,
}
