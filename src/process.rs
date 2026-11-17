use anyhow::{bail, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::{Pid, System};

#[derive(Debug, Clone, Serialize)]
pub struct ProcessDetail {
    pub pid: u32,
    pub name: String,
    pub exe: Option<PathBuf>,
    pub cwd: Option<PathBuf>,
    pub cmd: Vec<String>,
    pub memory_bytes: u64,
    pub git_branch: Option<String>,
    pub is_docker: bool,
}

pub fn inspect_process(sys: &System, pid: u32) -> Option<ProcessDetail> {
    let sys_pid = Pid::from(pid as usize);
    let proc = sys.process(sys_pid)?;

    let name = proc.name().to_string_lossy().to_string();
    let exe = proc.exe().map(Path::to_path_buf);
    let cwd = proc.cwd().map(Path::to_path_buf);
    let cmd = proc
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let memory_bytes = proc.memory();

    let git_branch = cwd.as_deref().and_then(find_git_branch);

    let is_docker = name.to_lowercase().contains("docker")
        || name.to_lowercase().contains("containerd")
        || cmd.iter().any(|arg| arg.to_lowercase().contains("docker"));

    Some(ProcessDetail {
        pid,
        name,
        exe,
        cwd,
        cmd,
        memory_bytes,
        git_branch,
        is_docker,
    })
}

pub fn terminate_pid(sys: &mut System, pid: u32) -> Result<()> {
    let sys_pid = Pid::from(pid as usize);
    if let Some(proc) = sys.process(sys_pid) {
        if proc.kill() {
            Ok(())
        } else {
            bail!("failed to send kill signal to PID {pid}")
        }
    } else {
        bail!("PID {pid} not found in active processes")
    }
}

pub fn find_git_branch(start_dir: &Path) -> Option<String> {
    let mut current = start_dir.to_path_buf();
    loop {
        let git_dir = current.join(".git");
        if git_dir.is_dir() {
            let head_file = git_dir.join("HEAD");
            if let Ok(content) = fs::read_to_string(head_file) {
                let trimmed = content.trim();
                if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
                    return Some(branch.to_string());
                } else if trimmed.len() >= 7 {
                    return Some(trimmed[..7].to_string());
                }
            }
            return None;
        } else if git_dir.is_file() {
            if let Ok(content) = fs::read_to_string(&git_dir) {
                if let Some(gitdir) = content.trim().strip_prefix("gitdir:") {
                    let head_file = current.join(gitdir.trim()).join("HEAD");
                    if let Ok(head) = fs::read_to_string(head_file) {
                        let trimmed = head.trim();
                        if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
                            return Some(branch.to_string());
                        }
                    }
                }
            }
            return None;
        }

        if !current.pop() {
            break;
        }
    }
    None
}
