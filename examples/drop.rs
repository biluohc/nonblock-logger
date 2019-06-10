#[macro_use]
extern crate log;
// #[macro_use]
extern crate nonblock_logger;

use nonblock_logger::{messages_in_channel, NonblockLogger};
use std::time;

fn main() {
    let now;

    {
        // use Drop wait for log thread
        let _handle = NonblockLogger::new()
        .log_to_stdout()
        .map_err(|e| eprintln!("failed to init nonblock_logger: {:?}", e))
        .unwrap();

        now = time::Instant::now();

        include!("log.snippet");

        println!("join0_{}: {:?}", messages_in_channel(), now.elapsed());
    }

    println!("join1_{}: {:?}", messages_in_channel(), now.elapsed());
}
