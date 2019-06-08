use chrono;
use log::{Level, Record};

use std::{fmt, mem, thread};

pub trait Formater: Send + Sync + 'static {
    fn boxed(self) -> Box<dyn Formater>;
    fn format(&self, &Record) -> String;
}

impl Formater for BaseFormater {
    fn boxed(self) -> Box<dyn Formater> {
        Box::new(self) as _
    }
    fn format(&self, record: &Record) -> String {
        const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S.%3f";

        let datetime = if self.local {
            chrono::Local::now().format(DATETIME_FORMAT)
        } else {
            chrono::Utc::now().format(DATETIME_FORMAT)
        };

        #[cfg(any(feature = "color"))]
        let level = self.color.colordfg(record.level(), AlignedLevel::new(record.level()));
        #[cfg(not(feature = "color"))]
        let level = AlignedLevel::new(record.level());

        format!(
            "{} {:5} [{}] ({}:{}) [{}] -- {}\n",
            datetime,
            level,
            current_thread_name(),
            record.file().unwrap_or("*"),
            record.line().unwrap_or(0),
            record.target(),
            record.args()
        )
    }
}

#[derive(Debug, Clone)]
pub struct BaseFormater {
    local: bool,
    #[cfg(any(feature = "color"))]
    color: ColoredLogConfig,
}

impl Default for BaseFormater {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseFormater {
    pub fn new() -> Self {
        Self {
            local: false,
            #[cfg(any(feature = "color"))]
            color: ColoredLogConfig::new(),
        }
    }
    pub fn local(mut self, local: bool) -> Self {
        self.local = local;
        self
    }
    #[cfg(any(feature = "color"))]
    pub fn color(self, color_: bool) -> Self {
        let Self { local, color } = self;
        let color = color.color(color_);
        Self { local, color }
    }
    #[cfg(any(feature = "color"))]
    pub fn colored(mut self, color: ColoredLogConfig) -> Self {
        self.color = color;
        self
    }
}

pub struct ThreadId(u64);

pub fn current_thread_name() -> &'static str {
    thread_local!(static THREAD_NAME: String = {
        let thread = thread::current();
        format!("{}.{}", unsafe { mem::transmute::<_, ThreadId>(thread.id()).0 }, thread.name()
        .map(|s| s.to_owned())
        // unamed thread, main has 4 chars, aligned
        .unwrap_or_else(||"****".to_owned()))
    });

    THREAD_NAME.with(|tname| unsafe { mem::transmute::<&str, &'static str>(tname.as_str()) })
}

#[derive(Debug, Clone, Copy)]
pub struct AlignedLevel(Level);

impl AlignedLevel {
    fn new(level: Level) -> Self {
        AlignedLevel(level)
    }
}

impl fmt::Display for AlignedLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:5}", self.0)
    }
}

#[cfg(any(feature = "color"))]
use self::color::ColoredLogConfig;
#[cfg(any(feature = "color"))]
pub mod color {
    use colored::Color;
    use log::Level;
    use std::fmt;

    pub struct ColoredFgWith<T> {
        text: T,
        color: Option<Color>,
    }

    impl<T> fmt::Display for ColoredFgWith<T>
    where
        T: fmt::Display,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if let Some(color) = self.color.as_ref() {
                write!(f, "\x1B[{}m{}\x1B[0m", color.to_fg_str(), self.text)
            } else {
                write!(f, "{}", self.text)
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ColoredLogConfig {
        error: Color,
        warn: Color,
        info: Color,
        debug: Color,
        trace: Color,
        color: bool,
    }

    impl Default for ColoredLogConfig {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ColoredLogConfig {
        #[inline]
        pub fn new() -> Self {
            Self {
                error: Color::BrightRed,
                warn: Color::Yellow,
                info: Color::Green,
                debug: Color::Cyan,
                trace: Color::BrightBlue,
                color: false,
            }
        }
        pub fn error(mut self, error: Color) -> Self {
            self.error = error;
            self
        }
        pub fn warn(mut self, warn: Color) -> Self {
            self.warn = warn;
            self
        }
        pub fn info(mut self, info: Color) -> Self {
            self.info = info;
            self
        }
        pub fn debug(mut self, debug: Color) -> Self {
            self.debug = debug;
            self
        }
        pub fn trace(mut self, trace: Color) -> Self {
            self.trace = trace;
            self
        }
        pub fn color(mut self, color: bool) -> Self {
            self.color = color;
            self
        }
        pub fn colordfg<T>(&self, level: Level, t: T) -> ColoredFgWith<T>
        where
            T: ColoredFg<T>,
        {
            t.colordfg(level, self)
        }
    }

    pub trait ColoredFg<T> {
        fn colordfg(self, level: Level, &ColoredLogConfig) -> ColoredFgWith<T>;
    }

    impl<T: fmt::Display> ColoredFg<T> for T {
        fn colordfg(self, level: Level, config: &ColoredLogConfig) -> ColoredFgWith<Self> {
            let color = if config.color {
                let colored = match level {
                    Level::Error => config.error,
                    Level::Warn => config.warn,
                    Level::Info => config.info,
                    Level::Debug => config.debug,
                    Level::Trace => config.trace,
                };
                Some(colored)
            } else {
                None
            };

            ColoredFgWith { color, text: self }
        }
    }
}
