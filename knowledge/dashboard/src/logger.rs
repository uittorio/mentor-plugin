use core::fmt;
use std::{
    collections::VecDeque,
    fmt::{Display, Formatter},
};

use chrono::{DateTime, Utc};

use crate::dates::now;

pub enum Log {
    Info(String, DateTime<Utc>),
}

impl Display for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let output = match self {
            Log::Info(m, date_time) => {
                format!("[INFO] {} {}", date_time.format("%d/%m/%Y %H:%M:%S"), m)
            }
        };

        write!(f, "{output}")
    }
}

pub struct DashboardLogger {
    pub logs: VecDeque<Log>,
}

impl DashboardLogger {
    pub fn new() -> Self {
        DashboardLogger {
            logs: VecDeque::new(),
        }
    }

    pub fn info(&mut self, msg: &str) {
        self.logs.push_front(Log::Info(msg.to_string(), now()))
    }
}
