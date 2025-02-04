use ratatui::crossterm::event::KeyEvent;

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

pub enum LogViewAction {
    ScrollUp,
    ScrollDown,
    ScrollToEnd,
    ScrollAuto,
}
