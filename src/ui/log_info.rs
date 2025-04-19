use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
use senpa::{Action, Dir, ProtoInfo};

use crate::TimestampedLog;

pub struct LogInfoPopup {
    pub border_style: Style,
    pub title_style: Style,
    pub style: Style,
    pub timestamped_log: TimestampedLog,
}

impl LogInfoPopup {
    pub fn new(timestamped_log: TimestampedLog) -> Self {
        Self {
            border_style: Style::default().yellow(),
            title_style: Style::default(),
            style: Style::default(),
            timestamped_log,
        }
    }
    fn get_content(&self) -> Text {
        let log = &self.timestamped_log.log;

        let mut content = vec![
            Line::from(vec![
                " Timestamp: ".bold(),
                format!("{}", self.timestamped_log.timestamp).into(),
            ]),
            Line::from(vec![
                " Action: ".bold(),
                match log.packet_filter.action {
                    Action::Pass => "Pass".green().bold(),
                    Action::Reject => "Reject".green().bold(),
                    Action::Block => "Block".red().bold(),
                },
            ]),
            Line::from(vec![
                "  Interface: ".bold(),
                log.packet_filter.interface.to_string().into(),
                "  Direction: ".bold(),
                match log.packet_filter.dir {
                    Dir::In => "Inbound".green(),
                    Dir::Out => "Outbound".blue(),
                },
            ]),
            Line::from(vec![
                "  Src IP: ".bold(),
                format!("{}", log.ip_data.src).into(),
            ]),
            Line::from(vec![
                "  Dst IP: ".bold(),
                format!("{}", log.ip_data.dst).into(),
            ]),
            Line::from(vec![
                "  IP packet length: ".bold(),
                format!("{}", log.ip_data.length).into(),
            ]),
            Line::from(vec![
                " 󰿘 Protocol: ".bold(),
                match &log.protocol.name {
                    senpa::ProtoName::Tcp => "TCP",
                    senpa::ProtoName::Udp => "UDP",
                    senpa::ProtoName::Other(s) => s,
                }.to_string().into(),
            ]),
        ];

        match &log.proto_info {
            ProtoInfo::UdpInfo(udp) => {
                content.push(Line::from(" 󱜠 UDP Info:".bold()));
                content.push(Line::from(vec![
                    "  Src Port: ".bold(),
                    format!("{}", udp.ports.srcport).into(),
                    " 󰣉 Dst Port: ".bold(),
                    format!("{}", udp.ports.dstport).into(),
                    " 󰏗 Data: ".bold(),
                    format!("{} bytes", udp.data_len).into(),
                ]));
            }

            ProtoInfo::TcpInfo(tcp) => {
                content.push(Line::from("  TCP Info:".bold()));
                content.push(Line::from(vec![
                    "  Src Port: ".bold(),
                    format!("{}", tcp.ports.srcport).into(),
                    " 󰣉 Dst Port: ".bold(),
                    format!("{}", tcp.ports.dstport).into(),
                    "  󰏗 Data: ".bold(),
                    format!("{} bytes", tcp.data_len).into(),
                ]));

                let mut line = vec![
                    "  Flags: ".bold(),
                    tcp.flags.to_string().into(),
                    "  󱎉 Seq #: ".bold(),
                    tcp.sequence_number.to_string().into(),
                ];
                if let Some(ack) = tcp.ack_number {
                    line.extend(vec![" 󰄬 Ack #: ".bold(), format!("{}", ack).into()]);
                }
                content.push(Line::from(line));

                let mut line = vec!["  Window: ".bold(), format!("{}", tcp.window).into()];
                if let Some(urg) = tcp.urg {
                    line.extend(vec!["  Urg: ".bold(), format!("{}", urg).into()]);
                }
                content.push(Line::from(line));

                content.push(Line::from(vec![
                    "  Options: ".bold(),
                    tcp.options.to_string().into(),
                ]));
            }

            ProtoInfo::UnknownInfo(info) => {
                content.push(Line::from("  Unknown Protocol Info:".bold()));
                content.push(Line::from(info.to_string()));
            }
        }

        Text::from(content)
    }
}

impl Widget for LogInfoPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = center(area, Constraint::Percentage(60), Constraint::Percentage(60));
        Clear.render(area, buf);

        let block = Block::new()
            .title(Line::from("  log info "))
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style);

        Paragraph::new(self.get_content())
            .wrap(Wrap { trim: true })
            .style(self.style)
            .block(block)
            .render(area, buf);
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
