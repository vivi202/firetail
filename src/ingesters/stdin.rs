use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use chrono::DateTime;
use rsyslog::Message;
use senpa::parse_log;
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    sync::Notify,
};

use crate::TimestampedLog;

use super::LogIngester;

pub struct StdinLogIngester {
    notify: Arc<Notify>,
    logs: Arc<RwLock<Vec<TimestampedLog>>>,
}

impl StdinLogIngester {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl LogIngester for StdinLogIngester {
    async fn ingest(&mut self) -> tokio::io::Result<()> {
        let stdin = stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        let logs = self.logs.clone();
        while let Some(raw_log) = lines.next_line().await? {
            let rsys_log: Result<Message, _> = rsyslog::Message::parse(&raw_log);

            if let Ok(msg) = rsys_log {
                let opnsense_raw_log = msg.msg.msg;

                if let Ok(flog) = parse_log(opnsense_raw_log) {
                    let timestamped_log = TimestampedLog {
                        log: flog,
                        timestamp: DateTime::from_str(msg.timestamp.unwrap()).unwrap(),
                    };

                    if let Ok(mut lock_guard) = logs.write() {
                        lock_guard.push(timestamped_log);
                    }

                    //notify packet filter
                    self.notify.notify_one();
                }
            }
        }
        Ok(())
    }

    fn get_notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    fn get_logs(&self) -> Arc<std::sync::RwLock<Vec<TimestampedLog>>> {
        self.logs.clone()
    }
}
