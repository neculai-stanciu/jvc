#![cfg(unix)]

use std::{
    io::{BufRead, BufReader, Error, ErrorKind, Result},
    process,
};

use log::debug;
use process::Stdio;

use crate::shell::{bash::Bash, powershell::PowerShell, shell::Shell, zsh::Zsh};

#[derive(Debug)]
struct ProcInfo {
    parent_pid: Option<u32>,
    command: String,
}

pub fn detect_shell() -> Option<Box<dyn Shell>> {
    let mut pid = Some(process::id());
    let mut visited = 0u8;

    while pid != None && visited < 10 {
        let proc_info: ProcInfo = get_proc_info(pid.unwrap()).ok()?;
        let binary = proc_info
            .command
            .trim_start_matches('-')
            .split('/')
            .last()
            .expect("Cannot read file name of process tree");
        match binary {
            "sh" | "bash" => return Some(Box::new(Bash)),
            "zsh" => return Some(Box::new(Zsh)),
            "pwsh" => return Some(Box::new(PowerShell)),
            cmd => debug!("binary is not a supported shell {:?}", cmd),
        }
        pid = proc_info.parent_pid;
        visited = visited + 1;
    }

    None
}

fn get_proc_info(pid: u32) -> Result<ProcInfo> {
    use std::process::Command;

    let stream = Command::new("ps")
        .arg("-o")
        .arg("ppid,comm")
        .arg(pid.to_string())
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::from(ErrorKind::UnexpectedEof))?;

    let mut output_buf_reader = BufReader::new(stream).lines();

    // skip next line
    output_buf_reader
        .next()
        .ok_or_else(|| Error::from(ErrorKind::UnexpectedEof))??;

    let line = output_buf_reader
        .next()
        .ok_or_else(|| Error::from(ErrorKind::NotFound))??;

    let mut parts = line.trim().split_whitespace();
    let ppid = parts
        .next()
        .expect("Can't read the ppid from ps. Expected pid to be first item in table.");
    let command = parts
        .next()
        .expect("Can't read the ppid from ps. Expected command to be second item in table");
    Ok(ProcInfo {
        parent_pid: u32::from_str_radix(ppid, 10).ok(),
        command: command.into(),
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    #[test]
    fn test_get_proc_info() {
        let subprocess = Command::new("bash")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Can't execute command");
        let process_info = get_proc_info(subprocess.id());
        let parent_pid = process_info.ok().and_then(|x| x.parent_pid);
        assert_eq!(parent_pid, Some(std::process::id()));
    }
}
