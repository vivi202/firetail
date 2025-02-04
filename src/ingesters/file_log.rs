use chrono::DateTime;
use rsyslog::Message;
use std::{
    io::{self},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, RwLock},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::Notify,
};

use senpa::parse_log;
use tokio::fs;

use crate::TimestampedLog;

use super::LogIngester;

pub struct FileLogIngester {
    pub logs: Arc<RwLock<Vec<TimestampedLog>>>,
    pub notify: Arc<Notify>,
    path: PathBuf,
}

impl LogIngester for FileLogIngester {
    async fn ingest(&mut self) -> io::Result<()> {
        let file = fs::File::open(&self.path).await?;

        let reader = BufReader::new(file);
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

    fn get_logs(&self) -> Arc<RwLock<Vec<TimestampedLog>>> {
        self.logs.clone()
    }
}
impl FileLogIngester {
    pub async fn new<T: AsRef<Path>>(path: T) -> Result<Self, io::Error> {
        match fs::try_exists(&path).await? {
            true => Ok(Self {
                logs: Arc::new(RwLock::new(Vec::new())),
                notify: Arc::new(Notify::new()),
                path: path.as_ref().to_path_buf(),
            }),
            false => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("the file {} doesn't exist", path.as_ref().to_str().unwrap()),
            )),
        }
    }
}
