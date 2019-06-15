use crate::NonblockLogger;
pub use log::Level;
pub use std::process;

pub fn wait_or_eprintln() {
    use std::{thread, time::Duration};

    if let Some(global) = NonblockLogger::global() {
        global.send_exit();
        while !global.exited() {
            thread::sleep(Duration::from_millis(1));
        }
    } else {
        eprintln!("use nonblock-logger::fatal! but NonblockLogger::global().is_none()");
    }
}

/// Logs a message at the error level and exit process with exit code 1.
#[macro_export]
macro_rules! fatal {
    (target: $target:expr, $($arg:tt)*) => (
        {
            log!(target: $target, $crate::macros::Level::Error, $($arg)*);
            $crate::macros::wait_or_eprintln();
            $crate::macros::process::exit(1);
        }
    );
    ($($arg:tt)*) => (
        {
            log!($crate::macros::Level::Error, $($arg)*);
            $crate::macros::wait_or_eprintln();
            $crate::macros::process::exit(1);
        }
    )
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     #[should_panic]
//     fn fatal_should_return_never() {
//         let _: usize = fatal!("fatal!() should return !");
//     }
// }
