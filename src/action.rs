use ratatui::crossterm::event::KeyEvent;
#[allow(clippy::enum_variant_names)]
pub enum Action {
    Quit,
    LogViewAction(LogViewAction),
    ToggleInfoPopup,
    DateSearchBegin,
    Edit(KeyEvent),
    EditDone,
    EditAbort,
    Tick,
}
#[allow(clippy::enum_variant_names)]
pub enum LogViewAction {
    ScrollUp,
    ScrollDown,
    ScrollToEnd,
    ScrollAuto,
}
