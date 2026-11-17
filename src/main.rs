mod net;
mod output;
mod process;

use anyhow::Result;
use clap::{Parser, Subcommand};
use net::SocketBinding;
use output::PortEntry;
use process::inspect_process;
use std::io::{self, Write};
use sysinfo::System;

#[derive(Parser)]
#[command(
    name = "portx",
    version,
    about = "Find, inspect, and free locked local ports"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Specific port(s) to inspect
    #[arg(value_name = "PORTS")]
    ports: Vec<u16>,

    /// Output results as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Include system and kernel internal ports
    #[arg(short, long, global = true)]
    all: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Terminate process(es) holding specified ports
    #[command(alias = "kill")]
    Free {
        /// Ports to free
        #[arg(required = true, value_name = "PORTS")]
        ports: Vec<u16>,

        /// Kill without confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// List all listening ports
    List {
        /// Include system and kernel internal ports
        #[arg(short, long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut sys = System::new_all();
    sys.refresh_all();

    match cli.command {
        Some(Commands::Free { ports, force }) => {
            handle_free(&mut sys, &ports, force)?;
        }
        Some(Commands::List { all }) => {
            handle_list(&sys, all, cli.json)?;
        }
        None => {
            if cli.ports.is_empty() {
                handle_list(&sys, cli.all, cli.json)?;
            } else if cli.ports.len() == 1 {
                handle_inspect_single(&sys, cli.ports[0], cli.json)?;
            } else {
                handle_inspect_multi(&sys, &cli.ports, cli.json)?;
            }
        }
    }

    Ok(())
}

fn handle_list(sys: &System, show_all: bool, json_output: bool) -> Result<()> {
    let bindings = net::list_listening_ports()?;
    let mut entries = collect_entries(sys, bindings);

    if !show_all {
        entries.retain(|e| e.pid.map_or(false, |p| p > 4));
    }
    entries.sort_by_key(|e| e.port);

    if json_output {
        println!("{}", output::render_json(&entries)?);
    } else if entries.is_empty() {
        println!("No listening ports detected");
    } else {
        println!("{}", output::render_table(&entries));
    }
    Ok(())
}

fn handle_inspect_single(sys: &System, port: u16, json_output: bool) -> Result<()> {
    let bindings = net::find_bindings_for_ports(&[port])?;
    let entries = collect_entries(sys, bindings);

    if entries.is_empty() {
        println!("No process is listening on port {port}");
        return Ok(());
    }

    if json_output {
        println!("{}", output::render_json(&entries)?);
    } else {
        for entry in &entries {
            println!("{}", output::render_detail(entry));
        }
    }
    Ok(())
}

fn handle_inspect_multi(sys: &System, ports: &[u16], json_output: bool) -> Result<()> {
    let bindings = net::find_bindings_for_ports(ports)?;
    let mut entries = collect_entries(sys, bindings);
    entries.sort_by_key(|e| e.port);

    if entries.is_empty() {
        println!("No processes found on specified ports");
        return Ok(());
    }

    if json_output {
        println!("{}", output::render_json(&entries)?);
    } else {
        println!("{}", output::render_table(&entries));
    }
    Ok(())
}

fn handle_free(sys: &mut System, ports: &[u16], force: bool) -> Result<()> {
    let bindings = net::find_bindings_for_ports(ports)?;
    if bindings.is_empty() {
        println!("No active processes found on specified ports");
        return Ok(());
    }

    for b in bindings {
        for pid in b.pids {
            let proc_name = sys
                .process(sysinfo::Pid::from(pid as usize))
                .map(|p| p.name().to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".into());

            if !force && !confirm_kill(b.port, pid, &proc_name) {
                println!("Skipped PID {pid} on port {}", b.port);
                continue;
            }

            match process::terminate_pid(sys, pid) {
                Ok(()) => println!("Killed PID {pid} ({proc_name}) on port {}", b.port),
                Err(err) => eprintln!("Failed to kill PID {pid} on port {}: {err}", b.port),
            }
        }
    }
    Ok(())
}

fn collect_entries(sys: &System, bindings: Vec<SocketBinding>) -> Vec<PortEntry> {
    let mut entries = Vec::new();
    for b in bindings {
        let ip_str = b.ip.to_string();
        if b.pids.is_empty() {
            entries.push(PortEntry {
                port: b.port,
                ip: ip_str,
                pid: None,
                process: None,
            });
        } else {
            for pid in b.pids {
                let proc_detail = inspect_process(sys, pid);
                entries.push(PortEntry {
                    port: b.port,
                    ip: ip_str.clone(),
                    pid: Some(pid),
                    process: proc_detail,
                });
            }
        }
    }
    entries
}

fn confirm_kill(port: u16, pid: u32, name: &str) -> bool {
    print!("Terminate PID {pid} ({name}) on port {port}? [y/N]: ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim().to_lowercase();
        trimmed == "y" || trimmed == "yes"
    } else {
        false
    }
}
