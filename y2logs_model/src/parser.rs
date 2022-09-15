use crate::log::{Entry, Level, Location};
use std::str::FromStr;
use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use thiserror::Error;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{char, digit1},
    character::complete::{newline, not_line_ending},
    combinator::{eof, map, map_res, opt, peek, recognize},
    multi::{many0, many_till},
    sequence::{delimited, pair, tuple},
    IResult
};

#[derive(Error, Debug)]
#[error("unable to parse the log")]
pub struct ParseError;

pub fn parse_string(s: &str) -> Result<Vec<Entry>, ParseError> {
    let mut parser = many0(parse_line);
    match parser(s) {
        Ok((_rest, entries)) => Ok(entries),
        // use anyhow to do not lose context
        Err(_e) => Err(ParseError)
    }
}

fn parse_line(s: &str) -> IResult<&str, Entry> {
    let mut parser = tuple((
        datetime,
        char(' '),
        level,
        char(' '),
        hostname,
        pid,
        char(' '),
        component,
        char(' '),
        parse_location,
        char(' '),
        parse_message,
    ));

    parser(s).map(|(rest, parts)| {
        let (datetime, _, level, _, hostname, pid, _, component, _, location, _, message) = parts;
        let entry = Entry {
            datetime,
            level,
            hostname,
            pid,
            component,
            location,
            message,
        };
        (rest, entry)
    })
}

fn date(s: &str) -> IResult<&str, NaiveDate> {
    let mut parser = tuple((number, char('-'), number, char('-'), number));
    parser(s).map(|(rest, (year, _, month, _, day))| {
        (
            rest,
            NaiveDate::from_ymd(year.try_into().unwrap(), month, day),
        )
    })
}

fn time(s: &str) -> IResult<&str, NaiveTime> {
    let mut parser = tuple((number, char(':'), number, char(':'), number));

    parser(s).map(|(rest, (hour, _, min, _, sec))| (rest, NaiveTime::from_hms(hour, min, sec)))
}

fn datetime(s: &str) -> IResult<&str, NaiveDateTime> {
    let mut parser = tuple((date, char(' '), time));

    parser(s).map(|(rest, (naive_date, _, naive_time))| {
        (rest, NaiveDateTime::new(naive_date, naive_time))
    })
}

fn hostname(s: &str) -> IResult<&str, String> {
    let parser = take_while(|c: char| c.is_alphanumeric() || c == '.' || c == '-');
    parser(s).map(|(rest, hostname)| (rest, hostname.to_string()))
}

fn pid(s: &str) -> IResult<&str, u32> {
    let mut parser = delimited(tag("("), number, tag(")"));
    parser(s).map(|(rest, value)| (rest, value))
}

fn component_name_parser(s: &str) -> IResult<&str, String> {
    let parser = take_while(|c: char| {
        c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '+' || c == ':'
    });
    parser(s).map(|(rest, name)| (rest, name.to_string()))
}

fn component(s: &str) -> IResult<&str, String> {
    let mut parser = delimited(tag("["), component_name_parser, tag("]"));
    parser(s).map(|(rest, value)| (rest, value))
}

fn digits_and_hyphen(s: &str) -> IResult<&str, &str> {
    recognize(pair(digit1, char('-')))(s)
}

fn parse_message(s: &str) -> IResult<&str, String> {
    let mut parser = many_till(
        recognize(pair(opt(newline), not_line_ending)),
        alt((
            map(pair(newline, peek(digits_and_hyphen)), |(_c, _s)| ""),
            eof,
        )),
    );
    parser(s).map(|(rest, value)| {
        let msg: String = value.0.into_iter().collect();
        (rest, msg)
    })
}

fn level(s: &str) -> IResult<&str, Level> {
    let mut parser = delimited(char('<'), number, char('>'));
    parser(s).map(|(rest, level)| (rest, Level::try_from(level).unwrap_or(Level::Unknown)))
}

fn parse_filename(s: &str) -> IResult<&str, String> {
    let parser = take_while(|c: char| c.is_alphanumeric() || c == '.' || c == '/' || c == '_');
    parser(s).map(|(rest, file)| (rest, file.to_string()))
}

fn parse_method(s: &str) -> IResult<&str, String> {
    let mut parser = delimited(char('('), take_until(")"), char(')'));
    parser(s).map(|(rest, method)| (rest, method.to_string()))
}

fn parse_line_number(s: &str) -> IResult<&str, u32> {
    let mut parser = pair(char(':'), number);
    parser(s).map(|(rest, (_, line))| (rest, line))
}

fn number(s: &str) -> IResult<&str, u32> {
    map_res(digit1, u32::from_str)(s)
}

fn parse_location(s: &str) -> IResult<&str, Location> {
    let mut parser = tuple((parse_filename, opt(parse_method), opt(parse_line_number)));
    parser(s).map(|(rest, (file, method, line))| (rest, Location { file, method, line }))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_y2log() {
        let y2log = r#"2022-08-25 14:28:44 <1> localhost.localdomain(12375) [libstorage] SystemCmd.cc(addLine):569 Adding Line 14...
Done
2022-08-25 14:28:44 <0> localhost.localdomain(12375) [libstorage] CmdParted.cc(parse):139 device:/dev/nvme0n1"#;
        let lines = parse_string(&y2log).unwrap();

        let datetime =
            NaiveDateTime::parse_from_str("2022-08-25 14:28:44", "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(
            lines[0],
            Entry {
                datetime,
                level: Level::Info,
                hostname: "localhost.localdomain".to_string(),
                pid: 12375,
                component: "libstorage".to_string(),
                location: Location {
                    file: "SystemCmd.cc".to_string(),
                    method: Some("addLine".to_string()),
                    line: Some(569)
                },
                message: "Adding Line 14...\nDone".to_string()
            }
        );
        assert_eq!(
            lines[1],
            Entry {
                datetime,
                level: Level::Debug,
                hostname: "localhost.localdomain".to_string(),
                pid: 12375,
                component: "libstorage".to_string(),
                location: Location {
                    file: "CmdParted.cc".to_string(),
                    method: Some("parse".to_string()),
                    line: Some(139)
                },
                message: "device:/dev/nvme0n1".to_string()
            }
        );
    }

    #[test]
    fn test_parse_line() {
        let line = String::from("2022-08-25 14:28:44 <1> localhost.localdomain(12375) [libstorage] SystemCmd.cc(addLine):569 Adding Line 14...");
        let (rest, entry) = parse_line(&line).unwrap();
        let datetime =
            NaiveDateTime::parse_from_str("2022-08-25 14:28:44", "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(
            entry,
            Entry {
                datetime,
                level: Level::Info,
                hostname: "localhost.localdomain".to_string(),
                pid: 12375,
                component: "libstorage".to_string(),
                location: Location {
                    file: "SystemCmd.cc".to_string(),
                    method: Some("addLine".to_string()),
                    line: Some(569)
                },
                message: "Adding Line 14...".to_string()
            }
        );
        assert_eq!(rest, "");
    }

    #[test]
    fn test_complete_location() {
        let str = "y2storage/storage_manager.rb(probe_performed):471";
        let (rest, location) = parse_location(&str).unwrap();
        assert_eq!(
            location,
            Location {
                file: "y2storage/storage_manager.rb".to_string(),
                method: Some("probe_performed".to_string()),
                line: Some(471)
            }
        );
        assert_eq!(rest, "");
    }

    #[test]
    fn test_simple_location() {
        let str = "YaPI";
        let (rest, location) = parse_location(&str).unwrap();
        assert_eq!(
            location,
            Location {
                file: "YaPI".to_string(),
                method: None,
                line: None
            }
        );
        assert_eq!(rest, "");
    }

    #[test]
    fn test_location_with_method() {
        let str = "modules/Stage.rb(Set)";
        let (rest, location) = parse_location(&str).unwrap();
        assert_eq!(
            location,
            Location {
                file: "modules/Stage.rb".to_string(),
                method: Some("Set".to_string()),
                line: None
            }
        );
        assert_eq!(rest, "");
    }

    #[test]
    fn test_location_with_line() {
        let str = "modules/Stage.rb:79";
        let (rest, location) = parse_location(&str).unwrap();
        assert_eq!(
            location,
            Location {
                file: "modules/Stage.rb".to_string(),
                method: None,
                line: Some(79)
            }
        );
        assert_eq!(rest, "");
    }
}
