pub use log::Level;
use NonblockLogger;

pub fn wait_and_exit() {
    use std::{process, thread, time::Duration};

    if let Some(global) = NonblockLogger::global() {
        global.send_exit();
        while !global.exited() {
            thread::sleep(Duration::from_millis(1));
        }
        process::exit(1)
    } else {
        panic!("use nonblock-logger::fatal! but NonblockLogger::global().is_none()");
    }
}

#[macro_export]
macro_rules! fatal {
    (target: $target:expr, $($arg:tt)*) => (
        {
            log!(target: $target, $crate::macros::Level::Error, $($arg)*);
            $crate::macros::wait_and_exit();
        }
    );
    ($($arg:tt)*) => (
        {
            log!($crate::macros::Level::Error, $($arg)*);
            $crate::macros::wait_and_exit();
        }
    )
}
