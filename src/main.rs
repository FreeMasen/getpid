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

fn get_processes() -> Result<Vec<Process>, Error> {
    // let processes = vec![];
    for res in WalkDir::new("/proc").min_depth(1).max_depth(1).follow_links(true) {
        let entry = res?;
        println!("{:?}, {:?}", entry, entry.file_type().is_dir());
        if entry.file_type().is_dir() {
            let name = entry.file_name().to_string_lossy();
            match name.parse::<usize>() {
                Ok(pid) => {
                    println!("found pid: {}", pid);
                    let comm = ::std::fs::read_to_string(entry.path().join("comm"))?;
                    let cmdline = ::std::fs::read_to_string(entry.path().join("cmdline"))?;
                    let exe_content = ::std::fs::read_to_string(entry.path().join("exe"))?;
                    let exe_data = Command::new(format!("stat {}", entry.path().join("exe").display())).output()?;
                    println!("info for {}", pid);
                    println!("----------");
                    println!("comm: {}", comm);
                    println!("cmdline: {}", cmdline);
                    println!("exe_data: {}", String::from_utf8_lossy(&exe_data.stdout));
                    println!("");
                },
                Err(e) => println!("parse error {}", e),
            }
        }
    }

    Ok(vec![])
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