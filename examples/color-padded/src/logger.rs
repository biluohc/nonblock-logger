use log::{LevelFilter, Record};
use nonblock_logger::{chrono, log};
use nonblock_logger::{current_thread_name, BaseFilter, BaseFormater, JoinHandle, NonblockLogger};

pub fn init(warn0_info1_debug2_trace3: u64) -> JoinHandle {
    let base_level = match warn0_info1_debug2_trace3 {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let other_level = if base_level <= LevelFilter::Info {
        base_level
    } else {
        LevelFilter::Info
    };

    let filter = BaseFilter::new()
        .max_level(base_level)
        .starts_with(true)
        .notfound(true)
        .chain_iter(vec![
            ("mio".to_owned(), other_level),
            ("tokio".to_owned(), other_level),
            ("hyper".to_owned(), other_level),
            ("want".to_owned(), other_level),
            ("tokio_tungstenite".to_owned(), other_level),
            ("tungstenite".to_owned(), other_level),
        ]);

    let formater = BaseFormater::new().formater(format);

    NonblockLogger::new()
        .quiet()
        .formater(formater)
        .filter(filter)
        .and_then(|l| l.log_to_stdout())
        .map_err(|e| eprintln!("failed to init non-block logger: {:?}", e))
        .unwrap()
}

pub fn format(_base: &BaseFormater, record: &Record) -> String {
    use chrono::prelude::*;
    use log::Level::*;
    use yansi::Paint;

    let lvl = StrPad::new(record.level().as_str());
    let level = match record.level() {
        Error => Paint::red(lvl),
        Warn => Paint::yellow(lvl),
        Info => Paint::green(lvl),
        Debug => Paint::cyan(lvl).dimmed(),
        Trace => Paint::blue(lvl).dimmed(),
    };

    current_thread_name(|ctn| {
        format!(
            "[{}{}{}:{:03} {}] {}\n",
            Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ"),
            level,
            record.module_path().unwrap_or("*"),
            // record.file().unwrap_or("*"),
            record.line().unwrap_or(0),
            ctn,
            record.args()
        )
    })
}

use std::fmt;
struct StrPad<'a> {
    str: &'a str,
}

impl<'a> StrPad<'a> {
    pub fn new(str: &'a str) -> Self {
        unsafe {
            Self {
                str: str.get_unchecked(..4),
            }
        }
    }
}

impl<'a> fmt::Display for StrPad<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " {} ", self.str)
    }
}
