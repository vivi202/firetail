use std::sync::{Arc, RwLock};

use ratatui::{
    layout::{Alignment, Constraint, Flex},
    prelude::{Buffer, Rect},
    style::{Style, Stylize},
    text::Text,
    widgets::{Cell, Row, StatefulWidget, Table, TableState},
};

use senpa::{Action, ProtoName};

use crate::TimestampedLog;

pub struct LogTable {
    pub logs: Arc<RwLock<Vec<TimestampedLog>>>,
    pub packet_window: usize,
    pub start: usize,
    pub selected_packet: usize,
    pub filtered_logs: Arc<RwLock<Vec<usize>>>,
}

impl LogTable {
    pub fn new(
        logs: Arc<RwLock<Vec<TimestampedLog>>>,
        filtered_logs: Arc<RwLock<Vec<usize>>>,
    ) -> Self {
        Self {
            logs,
            packet_window: 0,
            start: 0,
            selected_packet: 0,
            filtered_logs,
        }
    }
}

impl StatefulWidget for &mut LogTable {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let logs = self.logs.read().unwrap();
        self.packet_window = area.height.saturating_sub(1).into();
        let mut last_log = self.start.saturating_add(self.packet_window);

        if last_log <= self.selected_packet {
            last_log = self.selected_packet + 1;
            self.start = last_log
                .saturating_sub(self.packet_window)
                .min(self.selected_packet);
        }

        let filtered_logs = self.filtered_logs.read().unwrap();

        last_log = last_log.min(filtered_logs.len());

        state.select(Some(self.selected_packet.saturating_sub(self.start)));

        let rows: Vec<_> = filtered_logs[self.start..last_log]
            .iter()
            .map(|log_idx| {
                let x = &logs[*log_idx];
                let mut cells = vec![
                    Cell::new(Text::from(x.timestamp.naive_local().to_string()).centered()),
                    Cell::new(Text::from(x.log.packet_filter.interface.clone()).centered()),
                    Cell::new(Text::from(x.log.ip_data.src.to_string()).centered()),
                    Cell::new(Text::from(x.log.ip_data.dst.to_string()).centered()),
                ];

                cells.push(Cell::new(
                    Text::from(match &x.log.protocol.name {
                        ProtoName::Tcp => "tcp",
                        ProtoName::Udp => "udp",
                        ProtoName::Other(other) => other,
                    })
                    .centered(),
                ));

                Row::new(cells).style(match &x.log.packet_filter.action {
                    Action::Pass => Style::new().light_green(),
                    _ => Style::new().light_red(),
                })
            })
            .collect();

        let header = Row::new(
            ["Time", "Interface", "Source", "Destination", "Proto"]
                .iter()
                .map(|&c| Cell::from(Text::from(c).alignment(Alignment::Center))),
        );

        let table = Table::new(rows, [Constraint::Percentage(20); 5])
            .header(header)
            .flex(Flex::Center)
            .highlight_symbol(">>")
            .row_highlight_style(Style::new().on_gray());

        StatefulWidget::render(table, area, buf, state);
    }
}
