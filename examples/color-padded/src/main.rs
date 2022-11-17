#[macro_use]
extern crate nonblock_logger;

pub mod logger;

use nonblock_logger::messages_in_channel;
use std::time;

fn main() {
    let mut handle = logger::init(2);

    let now = time::Instant::now();

    include!("../../log.snippet");

    println!("join0_{}: {:?}", messages_in_channel(), now.elapsed());

    // wait for log thread
    handle.join();

    println!("join1_{}: {:?}", messages_in_channel(), now.elapsed());
}
