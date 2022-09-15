use std::path::PathBuf;
use clap::Args;
use y2logs_model::{Log, Level, Pid};

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

    let filtered = query.to_log();

    for line in filtered.entries {
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
    pub hostname: Option<String>
}
