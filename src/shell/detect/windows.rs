#![cfg(windows)]

use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
    path::PathBuf,
    process::{self, Command, Stdio},
};

use serde::Deserialize;

use crate::shell::{bash::Bash, powershell::PowerShell, shell::Shell, windows_cmd::WindowsCMD};
#[derive(Debug, Deserialize)]
pub struct ProcInfo {
    #[serde(rename = "ExecutablePath")]
    exec_path: Option<PathBuf>,
    #[serde(rename = "ParentProcessId")]
    p_pid: u32,
    #[serde(rename = "ProcessId")]
    pid: u32,
}

pub fn detect_shell() -> Option<Box<dyn Shell>> {
    let proc_map = get_proc_map().ok()?;
    let proc_tree = get_proc_tree(proc_map, process::id());

    for process in proc_tree {
        if let Some(exec_path) = process.exec_path {
            match exec_path.file_name().and_then(|x| x.to_str()) {
                Some("cmd.exe") => return Some(Box::from(WindowsCMD)),
                Some("bash.exe") => return Some(Box::new(Bash)),
                Some("powershell.exe") | Some("pwsh.exe") => return Some(Box::from(PowerShell)),
                _ => {}
            }
        }
    }

    None
}

type ProcessMap = HashMap<u32, ProcInfo>;

// write this more functional
fn get_proc_tree(mut proc_map: ProcessMap, pid: u32) -> Vec<ProcInfo> {
    let mut vec = vec![];
    let mut current = proc_map.remove(&pid);

    while let Some(proc) = current {
        current = proc_map.remove(&proc.p_pid);
        vec.push(proc)
    }

    vec
}

const WIN_COMMAND: &str = "wmic";
const WIN_COMMAND_ARGS: [&str; 4] = [
    "process",
    "get",
    "processid,parentprocessid,executablepath",
    "/format:csv",
];

fn get_proc_map() -> Result<ProcessMap> {
    let output_stream = Command::new(WIN_COMMAND)
        .args(&WIN_COMMAND_ARGS)
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or(Error::from(ErrorKind::UnexpectedEof))?;

    let mut reader = csv::Reader::from_reader(output_stream);
    let hashmap: HashMap<_, _> = reader
        .deserialize::<ProcInfo>()
        .filter_map(std::result::Result::ok)
        .map(|x| (x.pid, x))
        .collect();
    Ok(hashmap)
}

#[cfg(test)]
mod tests {
    use std::process;

    use super::get_proc_map;

    #[test]
    fn test_proc_map() {
        let procs = get_proc_map().unwrap();
        assert!(procs.contains_key(&process::id()))
    }
}
