#[macro_use]
extern crate log;
// #[macro_use]
extern crate nonblock_logger;

use nonblock_logger::{messages_in_channel, NonblockLogger};
use std::time;

fn main() {
    let mut handle = NonblockLogger::new()
        .log_to_stdout()
        .map_err(|e| eprintln!("failed to init nonblock_logger: {:?}", e))
        .unwrap();

    let now = time::Instant::now();

    include!("log.snippet");

    println!("join0_{}: {:?}", messages_in_channel(), now.elapsed());

    // let _: usize = fatal!("fatal!() will return !");

    // wait for log thread
    handle.join();

    println!("join1_{}: {:?}", messages_in_channel(), now.elapsed());
}
