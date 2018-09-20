extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;

use std::{
    fs::{
        read_link,
        read_to_string
    },
    io::Error as IoError,
};

use docopt::{Docopt,Error as DocError};
use walkdir::{WalkDir,Error as WalkError};

static HELP: &str = "
GET PID a tool for getting a pid for a running process.DocError

Usage:
    getpid <name>
    getpid [--help|-h]

Options:
    name       The name of the executable running
    --help -h  print this message
";
#[derive(Deserialize)]
struct Args {
    arg_name: String,
}

fn main() -> Result<(), Error> {
    let args: Args = Docopt::new(HELP)
                .and_then(|d| d.deserialize())
                .unwrap_or_else(|e| e.exit());
    if args.arg_name == String::new() {
        println!("{}", HELP);
        ::std::process::exit(0);
    }
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

fn get_processes() -> Result<Vec<Process>, Error> {
    // let processes = vec![];

    let ret = WalkDir::new("/proc").min_depth(1).max_depth(1).follow_links(true).into_iter().filter_map(|res| {
        println!("");
        if let Ok(entry) = res {
            if entry.file_type().is_dir() {
                if let Ok(pid) = entry.file_name().to_string_lossy().parse::<usize>() {
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

fn get_info_for(pid: usize) -> Option<Process> {
    let base = format!("/proc/{}", pid);
    let comm = get_str_for(&format!("{}/comm", base))?;
    let cmd_line = get_cmd_line(&format!("{}/cmdline", base))?;
    let exe = get_link_for(&format!("{}/exe", base))?;
    Some(Process {
        pid,
        cmd: comm,
        args: cmd_line,
        full_cmd_path: exe,
    })
}

fn get_cmd_line(path: &str) -> Option<Vec<String>> {
    let cmd_line = get_str_for(path)?;
    let mut all = cmd_line.split('\u{0}');
    let _comm = all.next();
    Some(all.map(String::from).collect())
}

fn get_str_for(path: &str) -> Option<String> {
    let ret = read_to_string(path).ok()?;
    Some(ret.trim().to_string())
}

fn get_link_for(path: &str) -> Option<String> {
    let link = read_link(path).ok()?;
    Some(link.to_string_lossy().to_string())
}


#[derive(Debug)]
struct Process {
    pub pid: usize,
    pub cmd: String,
    pub full_cmd_path: String,
    pub args: Vec<String>,
}


#[derive(Debug)]
enum Error {
    Doc(DocError),
    Io(IoError),
    Other(String),
    ParseInt(::std::num::ParseIntError),
    Walk(WalkError),
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