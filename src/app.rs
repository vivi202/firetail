use crate::{
    action::{self, Action},
    ui::{log_info::LogInfoPopup, log_view::LogView},
};
use action::LogViewAction::*;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, Paragraph},
    DefaultTerminal,
};

use std::{
    io,
    sync::{Arc, RwLock},
    time::Duration,
};

use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::TimestampedLog;
pub struct App {
    pub exit: bool,
    pub show_log_info_popup: bool,
    pub date_input: Input,
    action_rx: UnboundedReceiver<Action>,
    //Ui elements
    pub log_view: LogView,
}

#[derive(Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

impl App {
    pub fn new(
        logs: Arc<RwLock<Vec<TimestampedLog>>>,
        filtered_logs: Arc<RwLock<Vec<usize>>>,
    ) -> Self {
        let (action_tx, action_rx) = unbounded_channel::<Action>();

        let app = Self {
            exit: false,
            date_input: Input::default(),
            show_log_info_popup: false,
            log_view: LogView::new(logs, filtered_logs),
            action_rx,
        };

        let tick_tx = action_tx.clone();

        //Tick event
        tokio::spawn(async move {
            //send tick event at 30Hz
            let mut interval = time::interval(Duration::from_millis(33));
            loop {
                interval.tick().await;
                tick_tx.send(Action::Tick).unwrap();
            }
        });

        App::run_event_listener(action_tx);

        app
    }

    pub async fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.exit = true,

            Action::LogViewAction(action) => {
                self.log_view.update(action);
            }

            Action::ToggleInfoPopup => {
                self.show_log_info_popup = !self.show_log_info_popup;
            }

            Action::DateSearchBegin => {
                self.date_input.reset();
            }

            Action::EditAbort => {
                self.date_input.reset();
            }

            Action::EditDone => {
                let selected_log = self.log_view.get_selected_log();
                if let Some(selected_log) = selected_log {
                    if let Ok(datetime) = App::parse_date_time(
                        selected_log.timestamp.date_naive(),
                        self.date_input.value().to_owned(),
                    ) {
                        self.log_view.select_closest_date(datetime);
                    } else {
                        self.date_input.reset();
                    }
                }
            }

            Action::Edit(key_event) => {
                self.date_input.handle_event(&Event::Key(key_event));
            }

            Action::Tick => {}
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(1), Constraint::Length(3)])
                    .split(frame.area());

                self.log_view.draw(frame, layout[0]);

                if self.show_log_info_popup {
                    if let Some(log) = self.log_view.get_selected_log() {
                        let popup = LogInfoPopup::new(log);
                        frame.render_widget(popup, frame.area());
                    }
                }

                let footer =
                    Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
                        .split(layout[1]);

                let date_search =
                    Paragraph::new(Text::from(format!("󰥌 {}", self.date_input.value())))
                        .block(Block::bordered().yellow())
                        .centered();

                frame.render_widget(date_search, footer[0]);

                // Footer with centered instructions
                let instructions =
                    Paragraph::new(Text::from("↑/↓: Scroll  | i: Show log info |  q: Quit"))
                        .centered()
                        .style(Style::default().fg(Color::Gray))
                        .block(Block::bordered());

                frame.render_widget(instructions, footer[1]);
            })?;

            let action = self.action_rx.recv().await;
            if let Some(action) = action {
                self.update(action).await;
            }
        }

        Ok(())
    }

    pub fn run_event_listener(action_tx: UnboundedSender<Action>) {
        tokio::spawn(async move {
            let mut input_mode = InputMode::default();

            loop {
                let maybe_event = event::read();

                if let Ok(event) = maybe_event {
                    match event {
                        Event::Key(key_event) => match input_mode {
                            InputMode::Normal => match key_event.code {
                                KeyCode::Char('q') => {
                                    action_tx.send(Action::Quit).unwrap();
                                    break;
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    action_tx.send(Action::LogViewAction(ScrollUp)).unwrap()
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    action_tx.send(Action::LogViewAction(ScrollDown)).unwrap()
                                }
                                KeyCode::End => {
                                    action_tx.send(Action::LogViewAction(ScrollToEnd)).unwrap()
                                }
                                KeyCode::Char('.') => {
                                    action_tx.send(Action::LogViewAction(ScrollAuto)).unwrap()
                                }
                                KeyCode::Char('i') => {
                                    action_tx.send(Action::ToggleInfoPopup).unwrap()
                                }
                                KeyCode::Char('d') => {
                                    action_tx.send(Action::DateSearchBegin).unwrap();
                                    input_mode = InputMode::Editing;
                                }
                                _ => {}
                            },

                            InputMode::Editing => match key_event.code {
                                KeyCode::Enter => {
                                    input_mode = InputMode::Normal;
                                    action_tx.send(Action::EditDone).unwrap();
                                }

                                KeyCode::Esc => {
                                    input_mode = InputMode::Normal;
                                    action_tx.send(Action::EditAbort).unwrap();
                                }
                                _ => {
                                    action_tx.send(Action::Edit(key_event)).unwrap();
                                }
                            },
                        },
                        Event::Resize(_, _) => {}

                        _ => {}
                    }
                }
            }
        });
    }

    fn parse_date_time(current: NaiveDate, date_string: String) -> Result<NaiveDateTime, ()> {
        match NaiveDateTime::parse_from_str(&date_string, "%Y-%m-%d  %H:%M:%S") {
            Ok(datetime) => Ok(datetime),
            Err(_) => {
                let time = NaiveTime::parse_from_str(&date_string, "%H:%M:%S");
                match time {
                    Ok(time) => Ok(current.and_time(time)),
                    Err(_) => Err(()),
                }
            }
        }
    }
}
