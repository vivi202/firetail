#![doc = include_str!("../README.md")]
use app::App;
use chrono::Local;
use clap::Parser;
use cli::Cli;
use packet_filter::LogFilter;
use senpa::FwLog;
use std::{
    io::{self},
    process::exit,
};
mod cidr;
mod cli;
mod ingesters;
mod port_filter;
mod filter;
mod packet_filter;
mod ui;
mod action;
mod app;
use ingesters::{file_log::FileLogIngester, stdin::StdinLogIngester, LogIngester};

#[derive(Clone)]
pub struct TimestampedLog {
    pub timestamp: chrono::DateTime<Local>,
    pub log: FwLog,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let filter = match cli.build_filter() {
        Ok(filter) => filter,
        Err(e) => {
            println!("{:?}",e);
            exit(1);
        },
    };

    let (parsed_log, notify) = match cli.logfile {
        Some(log_file) => match FileLogIngester::new(log_file).await {
            Ok(mut ingester) => {
                let logs = ingester.get_logs();
                let notify = ingester.get_notify();
                tokio::spawn(async move { ingester.ingest().await });
                (logs, notify)
            }
            Err(e) => {
                eprintln!("Error initializing log ingester: {}", e);
                exit(2);
            }
        },
        None => {
            let mut ingester = StdinLogIngester::new();
            let logs = ingester.get_logs();
            let notify = ingester.get_notify();
            tokio::spawn(async move { ingester.ingest().await });
            (logs, notify)
        }
    };

    let mut log_filter = LogFilter::new(parsed_log.clone(), notify.clone()).filter(filter);

    let filtered_logs = log_filter.get_filtered_logs();

    //Start log processing

    tokio::spawn(async move { log_filter.process().await });

    let mut terminal = ratatui::init();

    let mut app = App::new(parsed_log.clone(), filtered_logs.clone());

    let app_result = app.run(&mut terminal).await;

    ratatui::restore();

    app_result
}
