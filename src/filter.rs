use log::{Level, LevelFilter, Metadata, Record};
use std::cmp::{max, Ordering};
use Error;

pub trait Filter: Send + Sync + 'static {
    fn boxed(self) -> Result<Box<dyn Filter>, Error>;
    fn log(&self, &Record) -> bool;
    fn enabled(&self, metadata: &Metadata) -> bool;
    fn maxlevel(&self) -> LevelFilter;
}

impl Filter for BaseFilter {
    fn boxed(self) -> Result<Box<dyn Filter>, Error> {
        Ok(Box::new(self.built()?) as _)
    }
    fn log(&self, record: &Record) -> bool {
        self.check(record.target(), record.level())
    }
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.check(metadata.target(), metadata.level())
    }
    fn maxlevel(&self) -> LevelFilter {
        self.max_level
    }
}

impl BaseFilter {
    fn built(mut self) -> Result<Self, Error> {
        // dbg!(&self);
        self.filters.sort_by(|a, b| a.0.cmp(&b.0));
        // dbg!(&self);

        self.max_level = self.max_level_get();
        let filters_length = self.filters.len();

        let starts_with = self.starts_with;
        self.filters
            .dedup_by(|a, b| if starts_with { b.0.starts_with(&a.0) } else { b.0 == a.0 });

        // dbg!(&self);

        if filters_length > self.filters.len() {
            Err("dedup token effect")?;
        }

        Ok(self)
    }
    fn check(&self, target: &str, level: Level) -> bool {
        if let Some(idx) = self
            .filters
            .binary_search_by(|(t, _level)| {
                if self.starts_with && target.starts_with(t) {
                    Ordering::Equal
                } else {
                    t.as_str().cmp(target)
                }
            })
            .ok()
        {
            unsafe { self.filters.get_unchecked(idx).1 >= level }
        } else {
            self.notfound
        }
    }
}

#[derive(Debug, Clone)]
pub struct BaseFilter {
    filters: Vec<(String, LevelFilter)>,
    max_level: LevelFilter,
    starts_with: bool,
    notfound: bool,
}

impl Default for BaseFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseFilter {
    pub fn new() -> Self {
        Self {
            filters: vec![],
            max_level: LevelFilter::Trace,
            starts_with: false,
            notfound: true,
        }
    }

    pub fn notfound(mut self, pass: bool) -> Self {
        self.notfound = pass;
        self
    }

    pub fn starts_with(mut self, yes: bool) -> Self {
        self.starts_with = yes;
        self
    }

    pub fn chain_iter<I>(mut self, filters: I) -> Self
    where
        I: IntoIterator<Item = (String, LevelFilter)>,
    {
        filters.into_iter().for_each(|tl| self.filters.push(tl));
        self
    }

    pub fn chain<S: Into<String>>(mut self, target: S, level: LevelFilter) -> Self {
        self.filters.push((target.into(), level));
        self
    }

    pub fn max_level(mut self, max_level: LevelFilter) -> Self {
        self.max_level = max_level;
        self
    }
    // outer use as arg for Logger
    pub fn max_level_get(&self) -> LevelFilter {
        let has = self.max_level;
        if let Some(compute) = self.filters.iter().max_by(|a, b| a.1.cmp(&b.1)) {
            if self.notfound {
                return max(compute.1, has);
            } else {
                return compute.1;
            }
        }
        has
    }
    pub fn starts_with_get(&self) -> bool {
        self.starts_with
    }
}
