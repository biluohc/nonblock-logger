#[macro_use]
extern crate nonblock_logger;
extern crate chrono;

use nonblock_logger::{log::Record, messages_in_channel, BaseFormater, FixedLevel, NonblockLogger};

use std::time;

fn main() {
    let formater = BaseFormater::new().formater(format);

    let mut handle = NonblockLogger::new()
        .formater(formater)
        .log_to_stdout()
        .map_err(|e| eprintln!("failed to init nonblock_logger: {:?}", e))
        .unwrap();

    let now = time::Instant::now();

    include!("log.snippet");

    println!("join0_{}: {:?}", messages_in_channel(), now.elapsed());

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
