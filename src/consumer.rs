use log::LevelFilter;
use std::io::{stderr, stdout, BufWriter, Stderr, Stdout, Write};
use std::{fmt, fs::File};
use {Error, Receiver};

pub trait Consumer: Send + Sync + 'static {
    fn boxed(self) -> Result<Box<dyn Consumer>, Error>;
    fn consume(&mut self, Receiver);
}

impl Consumer for BaseConsumer {
    fn boxed(self) -> Result<Box<dyn Consumer>, Error> {
        Ok(Box::new(self) as _)
    }

    fn consume(&mut self, channel: Receiver) {
        for message in channel {
            if let Some(message) = message {
                for (level, ref mut w) in self.outputers.iter_mut() {
                    if *level >= message.level {
                        if let Err(e) = w.write_all(message.content.as_bytes()) {
                            panic!("failed write log to {}: {}", (&*w).desc(), e);
                        }
                    }
                }
            } else {
                break;
            }
        }
    }
}

#[derive(Default)]
pub struct BaseConsumer {
    outputers: Vec<(LevelFilter, Box<dyn Outputer>)>,
}

impl fmt::Debug for BaseConsumer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BaseConsumer")
            .field(
                "outputers",
                &self.outputers.iter().map(|(l, o)| (l, o.desc())).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl BaseConsumer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chain<O: Outputer>(mut self, level: LevelFilter, outputer: O) -> Result<Self, Error> {
        self.outputers.push((level, outputer.boxed()?));
        Ok(self)
    }

    pub fn stdout(level: LevelFilter) -> Self {
        Self::new().chain(level, stdout()).unwrap()
    }

    pub fn stderr(level: LevelFilter) -> Self {
        Self::new().chain(level, stderr()).unwrap()
    }

    pub fn file(level: LevelFilter, file: File) -> Self {
        Self::new().chain(level, file).unwrap()
    }

    pub fn bufwriter<W>(level: LevelFilter, bufwriter: BufWriter<W>) -> Self
    where
        W: Write + Send + Sync + 'static,
    {
        Self::new().chain(level, bufwriter).unwrap()
    }
}

pub trait Outputer: Write + Send + Sync + 'static {
    fn boxed(self) -> Result<Box<dyn Outputer>, Error>;
    fn desc(&self) -> &str;
}

impl Outputer for Stdout {
    fn boxed(self) -> Result<Box<dyn Outputer>, Error> {
        Ok(Box::new(self) as _)
    }

    fn desc(&self) -> &str {
        "stdout"
    }
}

impl Outputer for Stderr {
    fn boxed(self) -> Result<Box<dyn Outputer>, Error> {
        Ok(Box::new(self) as _)
    }

    fn desc(&self) -> &str {
        "stderr"
    }
}

impl Outputer for File {
    fn boxed(self) -> Result<Box<dyn Outputer>, Error> {
        Ok(Box::new(self) as _)
    }

    fn desc(&self) -> &str {
        "file"
    }
}

impl<W> Outputer for BufWriter<W>
where
    W: Write + Send + Sync + 'static,
{
    fn boxed(self) -> Result<Box<dyn Outputer>, Error> {
        Ok(Box::new(self) as _)
    }

    fn desc(&self) -> &str {
        "bufwriter"
    }
}
