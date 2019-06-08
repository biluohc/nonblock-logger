#[macro_use]
extern crate log;

use log::LevelFilter;
use nonblock_logger::{messages_in_channel, BaseFilter, BaseFormater, BaseOutputer, JoinHandle, NonblockLogger};

use std::{fs::OpenOptions, io, time};

fn log() -> JoinHandle {
    let formater = BaseFormater::new().local(true).color(true).level(4);
    println!("{:?}", formater);

    let filter = BaseFilter::new()
        .starts_with(true)
        // .notfound(false)
        .chain("logs", LevelFilter::Trace)
        .chain("logt", LevelFilter::Off);
    println!("{:?}", filter);

    let outputer = BaseOutputer::stdout(filter.max_level_get())
        .chain(
            LevelFilter::Info,
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open("logtest.txt")
                .unwrap(),
        )
        .unwrap()
        .chain(LevelFilter::Error, io::stderr())
        .unwrap();
    println!("{:?}", outputer);

    let logger = NonblockLogger::new()
        // ::with_capacity(65536)
        .formater(formater)
        .filter(filter)
        .outputer(outputer);
    println!("{:?}", logger);    

    logger.spawn()
        .map_err(|e| eprintln!("failed to init nonblock_logger: {:?}", e))
        .unwrap()
}

fn main() {
    let handle = log();

    let now = time::Instant::now();

    include!("../../log.snippet");

    println!("join0_{}: {:?}", messages_in_channel(), now.elapsed());

    // wait for log thread
    handle.join();

    println!("join1_{}: {:?}", messages_in_channel(), now.elapsed());
}
