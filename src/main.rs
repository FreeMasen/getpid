extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;

use std::{
    collections::HashMap,
    io::Error as IoError,
    process::{
        Command,
    },
};

use docopt::{Docopt,Error as DocError};
use walkdir::{WalkDir,Error as WalkError, DirEntry};

static HELP: &str = "
GETPID a tool for getting a pid for a running process.DocError

Usage:
    getpid <name>

Options:
    name  The name of the executable running
";
#[derive(Deserialize)]
struct Args {
    arg_name: String,
}

fn main() -> Result<(), Error> {
    let args: Args = Docopt::new(HELP)
                .and_then(|d| d.deserialize())?;
    let processes = get_processes()?;
    let matches: Vec<Process> = processes.into_iter().filter(|p| p.cmd == args.arg_name).collect();
    if matches.len() > 1 {
        Err(Error::Other(format!("more than one process with the name {}", args.arg_name)))
    } else if matches.len() < 1 {
        Err(Error::Other(format!("no process found for {}", args.arg_name)))
    } else {
        println!("{}", matches[0].pid);
        Ok(())
    }
}

fn get_processes() -> Result<Vec<(usize, String, String, String)>, Error> {
    // let processes = vec![];

    let ret = WalkDir::new("/proc").min_depth(1).max_depth(1).follow_links(true).iter().filter_map(|res| {
        if let Ok(entry) = res {
            if entry.file_type().is_dir() {
                if let Ok(pid) = entry.file_name().parse::<usize>() {
                    get_info_for(pid)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }).collect();
    Ok(ret)
}

fn get_info_for(pid: usize) -> Option<(usize, String, String, String)> {
    if pid == 1 {
        return None
    }
    let base = format!("/proc/{}", pid);
    let comm = get_str_for(&format!("{}/comm", base))?;
    let cmd_line = get_str_for(&format!("{}/cmdline", base))?;
    let exe = get_link_for(&format!("{}/exe", base))?;
    Some((pid, comm, cmd_line, exe))
}

fn get_str_for(path: &str) -> Option<String> {
    ::std::fs::read_to_string(path).ok()
}

fn get_link_for(path: &str) -> Option<String> {
    let output = Command::new("stat").arg(path).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let first_line = text.lines().next()?;
    let link = first_line.trim_left_matches(format!("File: '{}' -> "));
    Some(link.trim_matches("'").to_string())
}



struct Process {
    pub pid: String,
    pub cmd: String,
    pub args: String,
}


#[derive(Debug)]
enum Error {
    Doc(DocError),
    Io(IoError),
    Other(String),
    ParseInt(::std::num::ParseIntError),
    Walk(WalkError),
}

impl Error {
    pub fn other(s: &str) -> Self {
        Error::Other(s.to_string())
    }
}

use std::error::Error as STDError;
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        if let Some(e) = self.cause() {
            return ::std::fmt::Display::fmt(e, f);
        }
        match self {
            Error::Other(s) => s.fmt(f),
            _ => unreachable!()
        }
    }
}

impl STDError for Error {
    fn cause(&self) -> Option<&STDError> {
        match self {
            Error::Doc(ref e) => Some(e),
            Error::Io(ref e) => Some(e),
            Error::ParseInt(ref e) => Some(e),
            Error::Walk(ref e) => Some(e),
            _ => None
        }
    }
}

impl From<IoError> for Error {
    fn from(other: ::std::io::Error) -> Self {
        Error::Io(other)
    }
}

impl From<DocError> for Error {
    fn from(other: DocError) -> Self {
        Error::Doc(other)
    }
}

impl From<WalkError> for Error {
    fn from(other: WalkError) -> Self {
        Error::Walk(other)
    }
}

impl From<::std::num::ParseIntError> for Error {
    fn from(other: ::std::num::ParseIntError) -> Self {
        Error::ParseInt(other)
    }
}