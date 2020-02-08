use chrono;
use log::{Level, Record};

use std::convert::AsRef;
use std::{fmt, mem, thread};

pub trait Formater: Send + Sync + 'static {
    fn boxed(self) -> Box<dyn Formater>;
    fn format(&self, record: &Record) -> String;
}

impl Formater for BaseFormater {
    fn boxed(self) -> Box<dyn Formater> {
        Box::new(self) as _
    }

    fn format(&self, record: &Record) -> String {
        self.formater_get()(self, record)
    }
}

pub fn format(base: &BaseFormater, record: &Record) -> String {
    let datetime = if base.local_get() {
        chrono::Local::now().format(base.datetime_get())
    } else {
        chrono::Utc::now().format(base.datetime_get())
    };

    #[cfg(any(feature = "color"))]
    let level = FixedLevel::with_color(record.level(), base.color_get())
        .length(base.level_get())
        .into_colored()
        .into_coloredfg();
    #[cfg(not(feature = "color"))]
    let level = FixedLevel::new(record.level()).length(base.level_get());

    format!(
        "{} {} [{}] ({}:{}) [{}] -- {}\n",
        datetime,
        level,
        current_thread_name(),
        record.file().unwrap_or("*"),
        record.line().unwrap_or(0),
        record.target(),
        record.args()
    )
}

impl fmt::Debug for BaseFormater {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(any(feature = "color"))]
        return fmt
            .debug_struct("BaseFormater")
            .field("local", &self.local)
            .field("level", &self.level)
            .field("datetime", &self.datetime)
            .field("color", &self.color)
            .finish();

        #[cfg(not(feature = "color"))]
        fmt.debug_struct("BaseFormater")
            .field("local", &self.local)
            .field("level", &self.level)
            .field("datetime", &self.datetime)
            .finish()
    }
}

// #[derive(Debug)]
pub struct BaseFormater {
    local: bool,
    level: usize,
    datetime: String,
    formater: Box<dyn Fn(&Self, &Record) -> String + Send + Sync + 'static>,
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
            level: 5,
            formater: Box::new(format) as _,
            datetime: "%Y-%m-%d %H:%M:%S.%3f".to_owned(),
            #[cfg(any(feature = "color"))]
            color: ColoredLogConfig::new(),
        }
    }

    pub fn local(mut self, local: bool) -> Self {
        self.local = local;
        self
    }

    #[inline]
    pub fn local_get(&self) -> bool {
        self.local
    }

    pub fn level(mut self, chars: usize) -> Self {
        self.level = chars;
        self
    }

    #[inline]
    pub fn level_get(&self) -> usize {
        self.level
    }

    pub fn datetime<S: Into<String>>(mut self, datetime: S) -> Self {
        self.datetime = datetime.into();
        self
    }

    #[inline]
    pub fn datetime_get(&self) -> &str {
        &self.datetime
    }

    pub fn formater<F>(mut self, formater: F) -> Self
    where
        F: Fn(&Self, &Record) -> String + Send + Sync + 'static,
    {
        self.formater = Box::new(formater) as _;
        self
    }

    #[inline]
    pub fn formater_get(&self) -> &(dyn Fn(&Self, &Record) -> String + Send + Sync + 'static) {
        &*self.formater
    }

    #[cfg(any(feature = "color"))]
    pub fn color(mut self, color_: bool) -> Self {
        self.color.color = color_;
        self
    }

    #[inline]
    #[cfg(any(feature = "color"))]
    pub fn color_get(&self) -> &ColoredLogConfig {
        &self.color
    }

    #[cfg(any(feature = "color"))]
    pub fn colored(mut self, color: ColoredLogConfig) -> Self {
        self.color = color;
        self
    }
}

pub fn current_thread_name() -> &'static str {
    struct ThreadId(u64);

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
pub struct FixedLevel {
    str: &'static str,
    length: usize,
    #[cfg(any(feature = "color"))]
    color: Option<Color>,
}

impl FixedLevel {
    #[inline]
    pub fn new(level: Level) -> Self {
        let str = match level {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO ",
            Level::Warn => "WARN ",
            Level::Error => "ERROR",
        };

        Self {
            str,
            length: 5,
            #[cfg(any(feature = "color"))]
            color: None,
        }
    }

    #[inline]
    pub fn length(mut self, length: usize) -> Self {
        debug_assert!(length <= 5);
        self.length = length;
        self
    }

    #[inline]
    #[cfg(any(feature = "color"))]
    pub fn with_color(level: Level, color_: &ColoredLogConfig) -> Self {
        let (str, color) = match level {
            Level::Trace => ("TRACE", color_.trace),
            Level::Debug => ("DEBUG", color_.debug),
            Level::Info => ("INFO ", color_.info),
            Level::Warn => ("WARN ", color_.warn),
            Level::Error => ("ERROR", color_.error),
        };

        Self {
            str,
            length: 5,
            color: if color_.color { Some(color) } else { None },
        }
    }

    #[cfg(any(feature = "color"))]
    pub fn into_colored(self) -> ColoredFixedLevel {
        ColoredFixedLevel::new(self)
    }
}

impl fmt::Display for FixedLevel {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.as_ref())
    }
}

impl AsRef<str> for FixedLevel {
    #[inline]
    fn as_ref(&self) -> &'static str {
        unsafe { self.str.get_unchecked(0..self.length) }
    }
}

#[cfg(any(feature = "color"))]
use self::color::{Color, ColoredFixedLevel, ColoredLogConfig};
#[cfg(any(feature = "color"))]
pub mod color {
    use super::FixedLevel;
    pub use colored::Color;
    use log::Level;
    use std::{fmt, mem};

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
        pub error: Color,
        pub warn: Color,
        pub info: Color,
        pub debug: Color,
        pub trace: Color,
        pub color: bool,
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
                color: true,
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

        pub fn coloredfg<T>(&self, level: Level, t: T) -> ColoredFgWith<T>
        where
            T: ColoredFg<T>,
        {
            t.coloredfg(level, self)
        }
    }

    pub trait ColoredFg<T> {
        fn coloredfg(self, level: Level, config: &ColoredLogConfig) -> ColoredFgWith<T>;
    }

    impl<T: fmt::Display> ColoredFg<T> for T {
        fn coloredfg(self, level: Level, config: &ColoredLogConfig) -> ColoredFgWith<Self> {
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

    // impl ColoredFg<ColoredFixedLevel> for ColoredFixedLevel {
    //     #[inline]
    //     fn coloredfg(self, _level: Level, _config: &ColoredLogConfig) -> ColoredFgWith<Self> {
    //         self.into_coloredfg()
    //     }
    // }
    // wait specialization for fixedLevel
    #[derive(Debug, Clone, Copy)]
    pub struct ColoredFixedLevel(FixedLevel);
    impl ColoredFixedLevel {
        #[inline]
        pub fn new(fl: FixedLevel) -> Self {
            ColoredFixedLevel(fl)
        }

        #[inline]
        pub fn into_coloredfg(mut self) -> ColoredFgWith<Self> {
            let color = mem::replace(&mut self.0.color, None);
            ColoredFgWith { color, text: self }
        }
    }
    impl fmt::Display for ColoredFixedLevel {
        #[inline]
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str(self.0.as_ref())
        }
    }
}
