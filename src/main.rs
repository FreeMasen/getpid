extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::{
    io::Error as IoError,
    process::{
        Command,
    },
};

use docopt::{Docopt,Error as DocError};

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
    let matches: Vec<Process> = processes.into_iter().filter(|p| p.cmd() == args.arg_name).collect();
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
    let output = Command::new("ps")
                        .arg("-o")
                        .arg("pid,args,comm")
                        .output()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text.lines().collect();
    let mut ret = Vec::with_capacity(lines.len() - 1);
    let mut first = true;
    let mut args_idx = 0;
    let mut comm_idx = 0;
    for line in lines {
        if first {
            let (a, c) = get_idxs(line)?;
            args_idx = a;
            comm_idx = c;
            first = false
        } else {
            ret.push(Process::try_parse(line, args_idx, comm_idx)?);
        }
    }
    Ok(ret)
}

fn get_idxs(line: &str) -> Result<(usize, usize), Error> {
    let args_start = line.find("ARGS").ok_or(Error::other("Unable to find ARGS in first line"))?;
    let comm_start = line.find("COMM").ok_or(Error::other("Unable to find COMM in first line"))?;
    Ok((args_start, comm_start))
}

struct Process {
    pub pid: String,
    pub cmd_path: String,
    pub args: String,
}

impl Process {
    pub fn try_parse(s: &str, args_idx: usize, comm_idx: usize) -> Result<Self, Error> {
        if s.len() - 1 < comm_idx {
            return Err(Error::other("invalid ps line: too short"));
        }
        let pid = String::from(s[0..args_idx].trim());
        let full_args = s[args_idx..comm_idx].trim();
        let cmd_path = String::from(s[comm_idx..].trim());
        let args = full_args.trim_left_matches(&cmd_path).to_string();
        Ok(Process {
            pid,
            cmd_path,
            args,
        })
    }

    pub fn cmd(&self) -> &str {
        self.cmd_path.split('/').last().unwrap_or("")
    }
}

#[derive(Debug)]
enum Error {
    Other(String),
    Io(IoError),
    Doc(DocError)
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
            Error::Io(ref e) => Some(e),
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