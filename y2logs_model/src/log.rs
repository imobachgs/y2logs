// TODO Query only needs the Log struct when executing the query

use chrono::naive::NaiveDateTime;
use std::{fmt, str::FromStr, path::Path, fs, error::Error};
use crate::parser;

/// Log level of an entry
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Fatal = 4,
    Unknown = 5,
}

/// TODO: use u8 instead of u32
impl TryFrom<u32> for Level {
    type Error = String;

    // Level::Error collides with Level::Error
    fn try_from(v: u32) -> Result<Self, <Level as TryFrom<u32>>::Error> {
        match v {
            0 => Ok(Level::Debug),
            1 => Ok(Level::Info),
            2 => Ok(Level::Warn),
            3 => Ok(Level::Error),
            4 => Ok(Level::Fatal),
            5 => Ok(Level::Unknown),
            _ => Err(format!("Could not convert {} into a level enum", v)),
        }
    }
}

impl FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(Level::Debug),
            "info" => Ok(Level::Info),
            "warn" => Ok(Level::Warn),
            "error" => Ok(Level::Error),
            "fatal" => Ok(Level::Fatal),
            "unknown" => Ok(Level::Unknown),
            _ => Err(format!("Could not convert {} into a level enum", s))
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Level::Debug => 0,
            Level::Info => 1,
            Level::Warn => 2,
            Level::Error => 3,
            Level::Fatal => 4,
            Level::Unknown => 5,
        };
        write!(f, "{}", value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Pid(pub u32);

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Pid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(value) => Ok(Pid(value)),
            Err(_) => Err(format!("Could not parse {}", s))
        }
    }
}

/// Represents the origin of a log entry
///
/// It might include the file, the method and the line (or almost any combination of them).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    /// File name
    pub file: String,
    /// Method name
    pub method: Option<String>,
    /// Line number
    pub line: Option<u32>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.file.clone())?;
        if let Some(method) = &self.method {
            write!(f, "({})", &method)?;
        }
        if let Some(line) = self.line {
            write!(f, ":{}", &line)?;
        }
        Ok(())
    }
}

/// Represents a log entry
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    /// Entry date and time
    pub datetime: NaiveDateTime,
    /// Entry level (debug, info, etc.)
    pub level: Level,
    /// Hostname
    pub hostname: String,
    /// Process ID
    pub pid: Pid,
    /// YaST2 component name
    pub component: String,
    /// Origin of the log message
    pub location: Location,
    /// Message body
    pub message: String,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} <{}> {}({}) [{}] {} {}",
            self.datetime,
            self.level,
            self.hostname,
            self.pid,
            self.component,
            self.location,
            self.message
        )
    }
}

/// Collection of YaST2 log entries
#[derive(Debug)]
pub struct Log {
    entries: Vec<Entry>
}

impl Log {
    /// Constructs a Log struct with the contents of a file
    pub fn from_file(file_path: &Path) -> Result<Self, Box<dyn Error>> {
        let contents = fs::read_to_string(file_path)?;
        match parser::parse_string(&contents) {
            Ok(entries) => Ok(Log { entries }),
            Err(e) => Err(Box::new(e))
        }
    }

    /// Constructs a query object for the current log
    pub fn query(&self) -> Query {
        Query::new(self)
    }
}

// supports "for line in log"
impl IntoIterator for Log {
    type Item = Entry;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

// supports "for line in &log"
impl<'a> IntoIterator for &'a Log {
    type Item = <std::slice::Iter<'a, Entry> as Iterator>::Item;
    type IntoIter = std::slice::Iter<'a, Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.as_slice().into_iter()
    }
}

/// Log query
///
/// Allows to build a query to filter out the content of a Log struct.
#[derive(Debug)]
pub struct  Query<'a> {
    log: &'a Log,
    level: Option<Level>,
    pid: Option<Pid>,
    component: Option<String>,
    hostname: Option<String>,
    from_datetime: Option<NaiveDateTime>,
    to_datetime: Option<NaiveDateTime>
}

impl<'a> Query<'a> {
    // Constructs a new query
    pub fn new(log: &'a Log) -> Self {
        Query {
            log,
            level: None,
            pid: None,
            component: None,
            hostname: None,
            from_datetime: None,
            to_datetime: None
        }
    }

    // Adds a condition on the level field
    pub fn with_level(&mut self, level: Level) -> &mut Self {
        self.level = Some(level);
        self
    }

    // Adds a condition on the pid field
    pub fn with_pid(&mut self, pid: Pid) -> &mut Self {
        self.pid = Some(pid);
        self
    }

    // Adds a condition on the component name field
    pub fn with_component(&mut self, component: String) -> &mut Self {
        self.component = Some(component);
        self
    }

    // Adds a condition on the hostname field
    pub fn with_hostname(&mut self, hostname: String) -> &mut Self {
        self.hostname = Some(hostname);
        self
    }

    pub fn from_datetime(&mut self, datetime: NaiveDateTime) -> &mut Self {
        self.from_datetime = Some(datetime);
        self
    }

    pub fn to_datetime(&mut self, datetime: NaiveDateTime) -> &mut Self {
        self.to_datetime = Some(datetime);
        self
    }

    // Filters the entries and constructs a new Log object with the result
    pub fn to_log(&self) -> Log {
        let entries = self.log.entries.iter()
            .filter(|e| {
                // https://github.com/rust-lang/rfcs/pull/2497
                if let Some(level) = self.level {
                    if level != e.level { return false };
                }

                if let Some(pid) = self.pid {
                    if pid != e.pid { return false };
                }

                if let Some(component) = &self.component {
                    if component != &e.component { return false };
                }

                if let Some(hostname) = &self.hostname {
                    if hostname != &e.hostname { return false };
                }

                if let Some(from_datetime) = &self.from_datetime {
                    if from_datetime > &e.datetime { return false };
                }

                if let Some(to_datetime) = &self.to_datetime {
                    if to_datetime < &e.datetime { return false };
                }

                true
            })
            .cloned()
            .collect();
        Log { entries }
    }
}
