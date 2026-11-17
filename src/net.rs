use anyhow::{Context, Result};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use std::collections::BTreeMap;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct SocketBinding {
    pub port: u16,
    pub ip: IpAddr,
    pub pids: Vec<u32>,
}

pub fn list_listening_ports() -> Result<Vec<SocketBinding>> {
    let af = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto = ProtocolFlags::TCP;

    let sockets = get_sockets_info(af, proto)
        .context("failed to query system network sockets")?;

    let mut bindings: BTreeMap<(u16, IpAddr), Vec<u32>> = BTreeMap::new();

    for s in sockets {
        if let ProtocolSocketInfo::Tcp(tcp) = s.protocol_socket_info {
            if tcp.state == TcpState::Listen {
                let entry = bindings.entry((tcp.local_port, tcp.local_addr)).or_default();
                for pid in s.associated_pids {
                    if !entry.contains(&pid) {
                        entry.push(pid);
                    }
                }
            }
        }
    }

    Ok(bindings
        .into_iter()
        .map(|((port, ip), pids)| SocketBinding { port, ip, pids })
        .collect())
}

pub fn find_bindings_for_ports(ports: &[u16]) -> Result<Vec<SocketBinding>> {
    let all = list_listening_ports()?;
    Ok(all.into_iter().filter(|b| ports.contains(&b.port)).collect())
}
