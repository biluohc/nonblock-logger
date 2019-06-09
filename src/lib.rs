extern crate chrono;
#[cfg(any(feature = "color"))]
extern crate colored;
extern crate crossbeam_channel;
extern crate log;

#[macro_use]
#[doc(hidden)]
pub mod macros;
mod consumer;
mod error;
mod filter;
mod formater;

pub use consumer::{BaseConsumer, Consumer, Outputer};
pub use error::Error;
pub use filter::{BaseFilter, Filter};
#[cfg(any(feature = "color"))]
pub use formater::color::{ColoredFg, ColoredFgWith, ColoredFixedLevel, ColoredLogConfig};
pub use formater::{current_thread_name, BaseFormater, FixedLevel, Formater};

use crossbeam_channel as channel;
use log::{set_logger, set_max_level, Level, Log, Metadata, Record, SetLoggerError};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{fmt, mem, thread};

static mut LOGGER: Option<NonblockLoggerGlobal> = None;

const NAME: &str = "log";

pub struct NonblockLogger {
    name: Option<String>,
    filter: Box<dyn Filter>,
    formater: Box<dyn Formater>,
    consumer: Option<Box<dyn Consumer>>,
    sendfn: Box<Fn(&Sender, Option<Message>) + Send + Sync + 'static>,
    sender: Sender,
    receiver: Option<Receiver>,
    exited: AtomicBool,
}

pub type Sender = channel::Sender<Option<Message>>;
pub type Receiver = channel::Receiver<Option<Message>>;

#[derive(Debug, Clone)]
pub struct Message {
    pub content: String,
    pub level: Level,
}

impl Message {
    pub fn new(content: String, level: Level) -> Self {
        Self { content, level }
    }
}

impl NonblockLogger {
    pub fn new() -> Self {
        let (mp, mc) = channel::unbounded();
        Self::new2(mp, mc)
    }
    pub fn with_capacity(cap: usize) -> Self {
        let (mp, mc) = channel::bounded(cap);
        Self::new2(mp, mc)
    }
    fn new2(mp: Sender, mc: Receiver) -> Self {
        Self {
            name: None,
            sender: mp,
            receiver: Some(mc),
            sendfn: Box::new(sendfn) as _,
            exited: AtomicBool::new(false),
            filter: BaseFilter::new().boxed().unwrap(),
            formater: BaseFormater::new().boxed(),
            consumer: Some(BaseConsumer::new().boxed().unwrap()),
        }
    }

    pub fn sendfn<F>(mut self, sendfn: F) -> Self
    where
        F: Fn(&Sender, Option<Message>) + Send + Sync + 'static,
    {
        self.sendfn = Box::new(sendfn) as _;
        self
    }

    pub fn formater<F: Formater>(mut self, formater: F) -> Self {
        self.formater = formater.boxed();
        self
    }

    pub fn filter<F: Filter>(mut self, filter: F) -> Result<Self, Error> {
        self.filter = filter.boxed()?;
        Ok(self)
    }

    pub fn consumer<C: Consumer>(mut self, consumer: C) -> Result<Self, Error> {
        self.consumer = Some(consumer.boxed()?);
        Ok(self)
    }

    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn name_get(&self) -> Option<&String> {
        self.name.as_ref()
    }

    fn log_to_channel(mut self) -> Result<Receiver, SetLoggerError> {
        let mc = mem::replace(&mut self.receiver, None).expect("NonblockLogger's receiver is None!");
        set_max_level(self.filter.maxlevel());
        let nob = NonblockLoggerGlobal(Arc::new(self));

        unsafe {
            LOGGER = Some(nob);
            assert!(LOGGER.is_some());
            let np = LOGGER.as_ref().unwrap() as _;
            set_logger(np)?;
        }

        Ok(mc)
    }

    pub fn spawn(mut self) -> Result<JoinHandle, Error> {
        let name = mem::replace(&mut self.name, None).unwrap_or_else(|| NAME.into());
        let mut consumer = mem::replace(&mut self.consumer, None).unwrap();
        let mc = self.log_to_channel()?;

        thread::Builder::new()
            .name(name)
            .spawn(move || {
                consumer.consume(mc);
                Self::global().map(|g| g.exit());
            })
            .map(|jh| JoinHandle::new(Self::global().unwrap(), jh))
            .map_err(Error::from)
    }
    pub fn log_to_stdout(mut self) -> Result<JoinHandle, Error> {
        let maxlevel = self.filter.maxlevel();
        self.consumer = Some(BaseConsumer::stdout(maxlevel).boxed()?);

        self.spawn()
    }
    pub fn log_to_stderr(mut self) -> Result<JoinHandle, Error> {
        let maxlevel = self.filter.maxlevel();
        self.consumer = Some(BaseConsumer::stderr(maxlevel).boxed()?);

        self.spawn()
    }
}

impl NonblockLogger {
    pub fn global() -> Option<&'static Self> {
        unsafe { LOGGER.as_ref().map(|g| g.0.as_ref()) }
    }
    pub fn send_exit(&self) {
        (*self.sendfn)(&self.sender, None)
    }
    pub fn exit(&self) {
        self.exited.store(true, Ordering::SeqCst)
    }
    pub fn exited(&self) -> bool {
        self.exited.load(Ordering::Relaxed)
    }
    pub fn messages_in_channel(&self) -> usize {
        self.sender.len()
    }
}

// if channel is full, send will block, but try_send don't
fn sendfn(sender: &Sender, msg: Option<Message>) {
    if msg.is_some() {
        sender.try_send(msg).expect("NonblockLogger send log message falied!")
    } else {
        sender.try_send(msg).expect("NonblockLogger send exit message falied!")
    }
}

pub fn messages_in_channel() -> usize {
    NonblockLogger::global().map(|g| g.messages_in_channel()).unwrap_or(0)
}

pub struct JoinHandle {
    logger: &'static NonblockLogger,
    join_handle: thread::JoinHandle<()>,
}

impl JoinHandle {
    fn new(logger: &'static NonblockLogger, join_handle: thread::JoinHandle<()>) -> Self {
        Self { logger, join_handle }
    }
    pub fn join(self) {
        self.logger.send_exit();
        self.join_handle.join().ok();
    }
}

struct NonblockLoggerGlobal(Arc<NonblockLogger>);

impl Log for NonblockLoggerGlobal {
    fn flush(&self) {}

    fn enabled(&self, metadata: &Metadata) -> bool {
        self.0.filter.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        let g = &self.0;

        if g.filter.log(record) {
            let content = g.formater.format(record);
            let message = Message::new(content, record.level());

            (*g.sendfn)(&g.sender, Some(message))
        }
    }
}

impl fmt::Debug for NonblockLogger {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("NonblockLogger")
            .field("name", &self.name)
            .field("exited", &self.exited)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
