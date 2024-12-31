const ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
use color_eyre::{eyre::bail, Result};
use pnet::datalink::{interfaces, NetworkInterface};
use regex::Regex;
use simple_dns::Packet;
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    collections::HashSet,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};
use tokio::net::UdpSocket;

fn get_iface(from: &SocketAddr, ifaces: &Vec<&NetworkInterface>) -> Result<String> {
    for i in ifaces {
        for ip in i.ips.iter() {
            if ip.contains(from.ip()) {
                return Ok(i.name.clone());
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

struct Rule {
    from: Regex,
    to: String,
    allow_questions: Regex,
    allow_answers: Regex,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut rules = Vec::new();
    rules.push(Rule {
        from: Regex::new("lan01")?,
        to: "lan01".to_string(),
        allow_answers: Regex::new(".*")?,
        allow_questions: Regex::new(".*")?,
    });
    let iface_reg = Regex::new(r"^lan.*$")?;

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    let addr = SocketAddrV4::new(ADDR, 5353).into();
    socket.bind(&addr)?;
    socket.join_multicast_v4(&ADDR, &Ipv4Addr::UNSPECIFIED)?;
    //socket.set_nonblocking(true)?;

    let interfaces = interfaces();
    let interfaces = interfaces
        .iter()
        .filter(|x| {
            x.is_up() && !x.is_loopback() && !x.ips.is_empty() && iface_reg.is_match(&x.name)
        })
        .collect();
    println!("{:?}", interfaces);
    let mut buf = [0; 1024];
    let socket = UdpSocket::from_std(socket.into())?;
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((_l, from)) => {
                let packet = Packet::parse(&buf)?;
                //println!("{:?} {:?}\n", from, packet);
                let iface = match get_iface(&from, &interfaces) {
                    Err(_) => {
                        eprintln!("Invalid packet received from {}", from);
                        continue;
                    }
                    Ok(name) => name,
                };
                let questions = get_questions(&packet);
                let answers = get_answers(&packet);
                println!(
                    "received packet on interface {} from {} questioning {:?} and answering {:?}",
                    iface, from, questions, answers
                );
                let mut out = HashSet::new();
                for r in &rules {
                    if r.from.is_match(&iface)
                        && (questions.iter().any(|x| r.allow_questions.is_match(x))
                            || answers.iter().any(|x| r.allow_answers.is_match(x)))
                    {
                        out.insert(r.to.clone());
                    }
                }
                out.remove(&iface);
                println!("relaying packet to {:?}", out);
                // socket.send_to(&buf, addr);
            }
            Err(_) => todo!(),
        }
    }
}
