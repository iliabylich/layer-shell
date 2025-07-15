#[derive(Debug)]
pub enum ControlEvent {
    ToggleSessionScreen,
    ReloadStyles,
    CapsLockToggled(ControlCapsLockToggledEvent),
    Exit,
}

#[derive(Debug)]
#[repr(C)]
pub struct ControlCapsLockToggledEvent {
    pub enabled: bool,
}
