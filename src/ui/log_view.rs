use crate::{
    action::{self},
    ui::log_table::LogTable,
    TimestampedLog,
};
use action::LogViewAction::*;
use chrono::NaiveDateTime;

use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, Scrollbar, ScrollbarOrientation, ScrollbarState, TableState},
    Frame,
};

use std::sync::{Arc, RwLock};
#[derive(Default, PartialEq, Eq)]
pub enum ScrollMode {
    #[default]
    Manual,
    Auto,
}
pub struct LogView {
    scrollbar_state: ScrollbarState,
    table_state: TableState,
    scroll_mode: ScrollMode,
    pub log_table: LogTable,
    pub filtered_logs: Arc<RwLock<Vec<usize>>>,
}

impl LogView {
    pub fn new(
        logs: Arc<RwLock<Vec<TimestampedLog>>>,
        filtered_logs: Arc<RwLock<Vec<usize>>>,
    ) -> Self {
        Self {
            table_state: TableState::new(),
            scrollbar_state: ScrollbarState::default(),
            log_table: LogTable::new(logs, filtered_logs.clone()),
            filtered_logs,
            scroll_mode: ScrollMode::default(),
        }
    }

    pub fn update(&mut self, action: action::LogViewAction) {
        match action {
            ScrollUp => self.scroll_up(),
            ScrollDown => self.scroll_down(),
            ScrollToEnd => self.scroll_to_end(),
            ScrollAuto => self.scroll_mode = ScrollMode::Auto,
        }
    }

    fn scroll_up(&mut self) {
        if self.table_state.selected() == Some(0) {
            self.log_table.start = self.log_table.start.saturating_sub(1);
        }
        self.log_table.selected_packet = self.log_table.selected_packet.saturating_sub(1);
        self.scrollbar_state.prev();
        self.table_state.scroll_up_by(1);
        if self.scroll_mode == ScrollMode::Auto {
            self.scroll_mode = ScrollMode::Manual;
        }
    }

    fn scroll_down(&mut self) {
        let log_len = self.filtered_logs.read().unwrap().len();

        if self.table_state.selected() == Some(self.log_table.packet_window.saturating_sub(1)) {
            self.log_table.start += 1;
            self.log_table.start = self.log_table.start.min(log_len.saturating_sub(1))
        }

        self.log_table.selected_packet =
            (self.log_table.selected_packet + 1).min(log_len.saturating_sub(1));

        self.scrollbar_state.next();
        self.table_state.scroll_down_by(1);

        if self.scroll_mode == ScrollMode::Auto {
            self.scroll_mode = ScrollMode::Manual;
        }
    }

    pub fn scroll_to_end(&mut self) {
        let end = self.filtered_logs.read().unwrap().len().saturating_sub(1);
        self.scrollbar_state = self.scrollbar_state.position(end);

        self.log_table.selected_packet = end;
    }

    pub fn select_log(&mut self, index: usize) {
        self.log_table.selected_packet = index;
        self.log_table.start = index;
        self.scrollbar_state = self.scrollbar_state.position(index);
        if self.scroll_mode == ScrollMode::Auto {
            self.scroll_mode = ScrollMode::Manual;
        }
    }

    pub fn get_selected_log(&self) -> Option<TimestampedLog> {
        let log_index = {
            let filtered_logs = self.filtered_logs.read().ok()?;
            filtered_logs.get(self.log_table.selected_packet).copied()?
        };

        let logs = self.log_table.logs.read().ok()?;
        logs.get(log_index).cloned()
    }

    pub fn select_closest_date(&mut self, date: chrono::NaiveDateTime) {
        let all_logs = self.log_table.logs.read().unwrap();
        let filtered_logs = self.filtered_logs.read().unwrap();

        let search_result = filtered_logs
            .binary_search_by(|index| all_logs[*index].timestamp.naive_local().cmp(&date));
        let filt_log_len = filtered_logs.len();

        drop(all_logs);
        drop(filtered_logs);

        let index = match search_result {
            Ok(index) => self.find_first_occurrence(&index),
            Err(index) => index,
        }
        .min(filt_log_len - 1);

        let mut index = self.find_first_occurrence(&index);

        let lower = index.saturating_sub(1);
        let lower_date = self.get_log_timestamp(lower);

        if date.signed_duration_since(lower_date).abs()
            < date
                .signed_duration_since(self.get_log_timestamp(index))
                .abs()
        {
            index = self.find_first_occurrence(&lower);
        }

        self.select_log(index.min(filt_log_len - 1));
    }

    fn get_log_timestamp(&self, index: usize) -> NaiveDateTime {
        let filtered_logs = self.filtered_logs.read().unwrap();
        let all_logs = self.log_table.logs.read().unwrap();
        all_logs[filtered_logs[index]].timestamp.naive_local()
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.scrollbar_state = self
            .scrollbar_state
            .content_length(self.filtered_logs.read().unwrap().len());
        if self.scroll_mode == ScrollMode::Auto {
            self.scroll_to_end();
        }
        let block = Block::default()
            .borders(Borders::ALL)
            .title("  Logs ")
            .style(Style::default());
        let inner_area = block.inner(area);

        frame.render_widget(block, area);

        frame.render_stateful_widget(&mut self.log_table, inner_area, &mut self.table_state);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .style(Style::new().yellow()),
            inner_area,
            &mut self.scrollbar_state,
        );
    }

    fn find_first_occurrence(&self, index: &usize) -> usize {
        let filtered_logs = self.filtered_logs.read().unwrap();
        let all_logs = self.log_table.logs.read().unwrap();

        let current_date = all_logs[filtered_logs[*index]].timestamp.naive_local();

        let mut new_index = *index;

        while new_index > 0
            && all_logs[filtered_logs[new_index]].timestamp.naive_local() >= current_date
        {
            new_index -= 1;
        }

        match new_index {
            0 => 0,
            _ => new_index + 1,
        }
    }
}
