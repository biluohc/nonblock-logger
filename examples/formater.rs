#[macro_use]
extern crate log;
extern crate chrono;
extern crate nonblock_logger;

use log::Record;
use nonblock_logger::{messages_in_channel, BaseFormater, FixedLevel, NonblockLogger};
use std::time;

fn main() {
    let formater = BaseFormater::new().formater(format);

    let handle = NonblockLogger::new()
        .formater(formater)
        .log_to_stdout()
        .map_err(|e| eprintln!("failed to init nonblock_logger: {:?}", e))
        .unwrap();

    let now = time::Instant::now();

    include!("log.snippet");

    println!("join0_{}: {:?}", messages_in_channel(), now.elapsed());

    // wait for log thread
    handle.join();

    println!("join1_{}: {:?}", messages_in_channel(), now.elapsed());
}

pub fn format(_: &BaseFormater, record: &Record) -> String {
    let level = FixedLevel::new(record.level()).length(4);

    // format!(
    //     "[{} {}#{}:{} {}] {}\n",
    //     chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
    //     level,
    //     record.module_path().unwrap_or("*"),
    //     // record.file().unwrap_or("*"),
    //     record.line().unwrap_or(0),
    //     nonblock_logger::current_thread_name(),
    //     record.args()
    // )

    format!(
        "{} [#{}:{}] {}\n",
        level,
        // record.module_path().unwrap_or("*"),
        record.file().unwrap_or("*"),
        record.line().unwrap_or(0),
        record.args()
    )
}
