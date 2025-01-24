use clap::{command, Parser};
use color_eyre::{eyre::bail, Result};
use pnet::datalink::{interfaces, NetworkInterface};
use pnet::ipnetwork::IpNetwork::{V4, V6};
use regex::Regex;
use serde::Deserialize;
use serde_json::from_reader;
use simple_dns::Packet;
use socket2::{Domain, Protocol, Socket, Type};
use std::fs::File;
use std::io::BufReader;
use std::net::UdpSocket;
use std::{
    collections::HashSet,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};
use tracing::{debug, error, trace};
use tracing_subscriber::EnvFilter;

const ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);

fn get_iface(from: &SocketAddr, ifaces: &Vec<Iface>) -> Result<String> {
    for i in ifaces {
        for ip in i.iface.ips.iter() {
            if ip.contains(from.ip()) {
                return Ok(i.iface.name.clone());
            }
        }
    }
    bail!("No interface found")
}

fn get_questions(packet: &Packet) -> HashSet<String> {
    packet
        .questions
        .iter()
        .map(|x| x.qname.to_string())
        .collect()
}
fn get_answers(packet: &Packet) -> HashSet<String> {
    packet.answers.iter().map(|x| x.name.to_string()).collect()
}

#[derive(Deserialize)]
struct Rule {
    #[serde(with = "serde_regex")]
    from: Regex,
    to: String,
    #[serde(with = "serde_regex", default)]
    allow_questions: Option<Regex>,
    #[serde(with = "serde_regex", default)]
    allow_answers: Option<Regex>,
}

#[derive(Debug)]
struct Iface {
    iface: NetworkInterface,
    socket: socket2::Socket,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the config file.
    #[arg(short, long)]
    config: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(with = "serde_regex")]
    interfaces: Regex,
    rules: Vec<Rule>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let cli = Cli::parse();
    let file = File::open(cli.config)?;
    let reader = BufReader::new(file);
    let config: Config = from_reader(reader)?;

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    let addr = SocketAddrV4::new(ADDR, 5353).into();
    socket.bind(&addr)?;
    socket.join_multicast_v4(&ADDR, &Ipv4Addr::UNSPECIFIED)?;

    let interfaces = interfaces();
    debug!("Found interface: {:?}", interfaces);
    let interfaces = interfaces
        .iter()
        .filter(|x| {
            x.is_up()
                && !x.is_loopback()
                && !x.ips.is_empty()
                && config.interfaces.is_match(&x.name)
        })
        .map(|x| -> Result<Iface> {
            let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
            socket.set_reuse_address(true)?;
            socket.set_reuse_port(true)?;
            for addr in &x.ips {
                if let V4(addr) = addr {
                    let sock_addr = SocketAddrV4::new(addr.ip(), 5353).into();
                    socket.bind(&sock_addr)?;
                }
            }
            Ok(Iface {
                iface: x.clone(),
                socket,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    debug!("Filtered interfaces: {:?}", interfaces);
    let mut buf = [0u8; 4096];
    let socket: UdpSocket = socket.into();
    loop {
        match socket.recv_from(&mut buf) {
            Ok((_l, from)) => {
                if interfaces
                    .iter()
                    .any(|x| x.iface.ips.iter().any(|y| y.ip() == from.ip()))
                {
                    continue;
                }
                let packet = Packet::parse(&buf)?;
                trace!("Packet rec: {:?}", packet);
                let iface = match get_iface(&from, &interfaces) {
                    Err(_) => {
                        error!("No interface found for packet received from {}", from);
                        continue;
                    }
                    Ok(name) => name,
                };
                let questions = get_questions(&packet);
                let answers = get_answers(&packet);
                debug!(
                    "received packet on interface {} from {} questioning {:?} and answering {:?}",
                    iface, from, questions, answers
                );
                let mut out = HashSet::new();
                for r in &config.rules {
                    if r.from.is_match(&iface)
                        && (r
                            .allow_questions
                            .as_ref()
                            .is_some_and(|r| questions.iter().any(|x| r.is_match(x)))
                            || (r
                                .allow_answers
                                .as_ref()
                                .is_some_and(|r| answers.iter().any(|x| r.is_match(x)))))
                    {
                        out.insert(r.to.clone());
                    }
                }
                out.remove(&iface);
                debug!("relaying packet to {:?}", out);
                for i in &interfaces {
                    if out.contains(&i.iface.name) {
                        let sock_addr = SocketAddrV4::new(ADDR, 5353).into();
                        i.socket.send_to(&buf, &sock_addr)?;
                    }
                }
            }
            Err(_) => todo!(),
        }
    }
}
