use chrono::naive::NaiveDateTime;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
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
    type Error = &'static str;

    // Level::Error collides with Level::Error
    fn try_from(v: u32) -> Result<Self, <Level as TryFrom<u32>>::Error> {
        match v {
            0 => Ok(Level::Debug),
            1 => Ok(Level::Info),
            2 => Ok(Level::Warn),
            3 => Ok(Level::Error),
            4 => Ok(Level::Fatal),
            5 => Ok(Level::Unknown),
            _ => Err("Could not convert {} into a level enum"),
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
#[derive(Debug, PartialEq, Eq)]
pub struct Location {
    pub file: String,
    pub method: Option<String>,
    pub line: Option<u32>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut content = self.file.clone();
        if let Some(method) = &self.method {
            content.push_str(&format!("({})", &method));
        }
        if let Some(line) = self.line {
            content.push_str(&format!(":{}", &line));
        }
        write!(f, "{}", content)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub datetime: NaiveDateTime,
    pub level: Level,
    pub hostname: String,
    pub pid: u32,
    pub component: String,
    pub location: Location,
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
