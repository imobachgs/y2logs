use std::path::PathBuf;
use clap::Args;
use y2logs_model::{Log, Level, Pid};
use chrono::{naive::NaiveDateTime, format::ParseResult};

pub fn run(args: &FilterArgs) {
    let log = Log::from_file(&args.file).unwrap();
    let mut query = log.query();

    if let Some(level) = &args.level {
        query.with_level(*level);
    }

    if let Some(pid) = &args.pid {
        query.with_pid(*pid);
    }

    if let Some(component) = &args.component {
        query.with_component(component.to_owned());
    }

    if let Some(hostname) = &args.hostname {
        query.with_hostname(hostname.to_owned());
    }

    if let Some(datetime) = &args.from_datetime {
        query.from_datetime(*datetime);
    }

    if let Some(datetime) = &args.to_datetime {
        query.to_datetime(*datetime);
    }

    let filtered = query.to_log();

    for line in filtered {
        println!("{}", line);
    }
}

#[derive(Args, Debug)]
pub struct FilterArgs {
    /// YaST2 logs file path
    pub file: PathBuf,
    /// Filter by level (debug, info, warn, error, fatal or unknown)
    #[clap(long)]
    pub level: Option<Level>,
    /// Filter by process ID
    #[clap(long)]
    pub pid: Option<Pid>,
    /// Filter by component name
    #[clap(long)]
    pub component: Option<String>,
    /// Filter by hostname
    #[clap(long)]
    pub hostname: Option<String>,
    /// From the given date/time
    #[clap(long,value_parser=parse_datetime)]
    pub from_datetime: Option<NaiveDateTime>,
    /// Up to the given date/time
    #[clap(long,value_parser=parse_datetime)]
    pub to_datetime: Option<NaiveDateTime>
}

// Parse datetime from the command line
//
// TODO: try multiple formats
fn parse_datetime(s: &str) -> ParseResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
}
