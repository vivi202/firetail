use std::sync::{Arc, RwLock};

use tokio::sync::Notify;

use crate::{filter::Filter, TimestampedLog};

pub struct LogFilter {
    all_packets: Arc<RwLock<Vec<TimestampedLog>>>,
    filter: Option<Filter>,
    //Contains the index of packet that comply with the filter.
    filtered_logs: Arc<RwLock<Vec<usize>>>,
    log_notify: Arc<Notify>,
    last_processed_packet: usize,
}

impl LogFilter {
    pub fn new(all_packets: Arc<RwLock<Vec<TimestampedLog>>>, log_notify: Arc<Notify>) -> Self {
        Self {
            filter: None,
            filtered_logs: Arc::new(RwLock::new(Vec::new())),
            log_notify,
            all_packets,
            last_processed_packet: 0,
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn get_filtered_logs(&self) -> Arc<RwLock<Vec<usize>>> {
        self.filtered_logs.clone()
    }

    //TODO allow to change filter in realtime.
    pub async fn process(&mut self) {
        loop {
            //Wait for new packets
            self.log_notify.notified().await;
            self.filter_logs();
        }
    }

    pub fn filter_logs(&mut self) {
        let logs = self.all_packets.read().unwrap();
        let start_index = self.last_processed_packet;
        let end_index = logs.len();

        for index in start_index..end_index {
            if let Some(filt) = &self.filter {
                if filt.test(&logs[index]) {
                    self.filtered_logs.write().unwrap().push(index);
                }
            } else {
                self.filtered_logs.write().unwrap().push(index);
            }
        }

        self.last_processed_packet = end_index;
    }
}
