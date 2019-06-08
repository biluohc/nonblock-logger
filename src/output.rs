use log::LevelFilter;
use std::io::{self, stderr, stdout, BufWriter, Stderr, Stdout, Write};
use std::{fmt, fs::File};
use Receiver;

pub trait Outputer: Send + Sync + 'static {
    fn boxed(self) -> Box<Outputer>;
    fn consumer_all(&mut self, Receiver);
}

impl Outputer for BaseOutputer {
    fn boxed(mut self) -> Box<Outputer> {
        // important should be in the front

        // dbg!(&self);
        self.outputs.sort_by(|a, b| a.0.cmp(&b.0));
        // dbg!(&self);

        Box::new(self) as _
    }
    fn consumer_all(&mut self, channel: Receiver) {
        for message in channel {
            if let Some(message) = message {
                for (level, ref mut w) in self.outputs.iter_mut() {
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

pub struct BaseOutputer {
    outputs: Vec<(LevelFilter, Box<dyn Output>)>,
}

impl fmt::Debug for BaseOutputer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BaseOutputer")
            .field(
                "outputs",
                &self.outputs.iter().map(|(l, o)| (l, o.desc())).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl BaseOutputer {
    pub fn new() -> Self {
        Self { outputs: vec![] }
    }
    pub fn chain<O: Output>(mut self, level: LevelFilter, output: O) -> io::Result<Self> {
        self.outputs.push((level, output.boxed()?));
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

pub trait Output: Write + Send + Sync + 'static {
    fn boxed(self) -> io::Result<Box<dyn Output>>;
    fn desc(&self) -> &str;
}

impl Output for Stdout {
    fn boxed(self) -> io::Result<Box<dyn Output>> {
        Ok(Box::new(self) as _)
    }
    fn desc(&self) -> &str {
        "stdout"
    }
}

impl Output for Stderr {
    fn boxed(self) -> io::Result<Box<dyn Output>> {
        Ok(Box::new(self) as _)
    }
    fn desc(&self) -> &str {
        "stderr"
    }
}

impl Output for File {
    fn boxed(self) -> io::Result<Box<dyn Output>> {
        Ok(Box::new(self) as _)
    }
    fn desc(&self) -> &str {
        "file"
    }
}

impl<W> Output for BufWriter<W>
where
    W: Write + Send + Sync + 'static,
{
    fn boxed(self) -> io::Result<Box<dyn Output>> {
        Ok(Box::new(self) as _)
    }
    fn desc(&self) -> &str {
        "bufwriter"
    }
}
