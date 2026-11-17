use crate::process::ProcessDetail;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use serde::Serialize;

#[derive(Serialize)]
pub struct PortEntry {
    pub port: u16,
    pub ip: String,
    pub pid: Option<u32>,
    pub process: Option<ProcessDetail>,
}

pub fn render_table(entries: &[PortEntry]) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("PORT").fg(Color::Cyan),
            Cell::new("PID").fg(Color::Yellow),
            Cell::new("PROCESS"),
            Cell::new("BRANCH"),
            Cell::new("MEMORY"),
            Cell::new("CWD"),
        ]);

    for entry in entries {
        let port_str = entry.port.to_string();
        let pid_str = entry
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".into());

        let (proc_name, branch, mem, cwd) = match &entry.process {
            Some(p) => {
                let name = if p.is_docker {
                    format!("{} [docker]", p.name)
                } else {
                    p.name.clone()
                };
                let branch = p.git_branch.clone().unwrap_or_else(|| "-".into());
                let mem = format!("{:.1} MB", p.memory_bytes as f64 / 1_048_576.0);
                let cwd = p
                    .cwd
                    .as_ref()
                    .map(|c| c.display().to_string())
                    .unwrap_or_else(|| "-".into());
                (name, branch, mem, cwd)
            }
            None => ("-".into(), "-".into(), "-".into(), "-".into()),
        };

        table.add_row(vec![
            Cell::new(port_str).fg(Color::Green),
            Cell::new(pid_str),
            Cell::new(proc_name),
            Cell::new(branch).fg(Color::Magenta),
            Cell::new(mem),
            Cell::new(cwd),
        ]);
    }

    table.to_string()
}

pub fn render_detail(entry: &PortEntry) -> String {
    let mut out = String::new();
    out.push_str(&format!("Port:        {} ({})\n", entry.port, entry.ip));
    if let Some(pid) = entry.pid {
        out.push_str(&format!("PID:         {}\n", pid));
    }
    if let Some(p) = &entry.process {
        out.push_str(&format!("Name:        {}\n", p.name));
        if p.is_docker {
            out.push_str("Container:   Yes (Docker-managed)\n");
        }
        out.push_str(&format!(
            "Memory:      {:.1} MB\n",
            p.memory_bytes as f64 / 1_048_576.0
        ));
        if let Some(branch) = &p.git_branch {
            out.push_str(&format!("Git Branch:  {}\n", branch));
        }
        if let Some(cwd) = &p.cwd {
            out.push_str(&format!("CWD:         {}\n", cwd.display()));
        }
        if let Some(exe) = &p.exe {
            out.push_str(&format!("Executable:  {}\n", exe.display()));
        }
        if !p.cmd.is_empty() {
            out.push_str(&format!("Command:     {}\n", p.cmd.join(" ")));
        }
    } else {
        out.push_str("Process:     Unknown (system or permission denied)\n");
    }
    out
}

pub fn render_json(entries: &[PortEntry]) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(entries)?)
}
