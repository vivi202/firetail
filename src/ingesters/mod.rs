use std::sync::{Arc, RwLock};

use tokio::{io, sync::Notify};

use crate::TimestampedLog;

pub trait LogIngester {
    async fn ingest(&mut self) -> io::Result<()>;
    fn get_notify(&self) -> Arc<Notify>;
    fn get_logs(&self) -> Arc<RwLock<Vec<TimestampedLog>>>;
}

pub mod file_log;
pub mod stdin;
