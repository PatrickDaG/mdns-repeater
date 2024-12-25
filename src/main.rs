const ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
use color_eyre::Result;
use pnet::datalink::interfaces;
use regex::Regex;
use simple_dns::Packet;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<()> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    let addr = SocketAddrV4::new(Ipv4Addr::new(224, 0, 0, 251), 5353).into();
    socket.bind(&addr)?;
    socket.join_multicast_v4(&ADDR, &Ipv4Addr::new(10, 99, 10, 1))?;
    //socket.set_nonblocking(true)?;

    let iface_reg = Regex::new(r"^lan.*$")?;
    let interfaces = interfaces();
    let interfaces = interfaces.iter().filter(|x| {
        x.is_up() && !x.is_loopback() && !x.ips.is_empty() && iface_reg.is_match(&x.name)
    });
    for i in interfaces {
        println!("{:?}", i);
    }

    let mut buf = [0; 1024];
    let socket = UdpSocket::from_std(socket.into())?;
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((_l, from)) => {
                let packet = Packet::parse(&buf)?;
                println!("{:?} {:?}\n", from, packet);
            }
            Err(_) => todo!(),
        }
    }
}
/*
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 5354))?;

} */
/*
 table ip mdns {
        chain prerouting {
            type filter hook prerouting priority mangle; policy accept;

            iifname lan01 ip daddr 224.0.0.251 meta mark 0xa5f3 jump mdns-saddr
            iifname lan01 ip daddr 224.0.0.251 meta mark != 0xa5f3 jump mdns
        }
        chain mdns {
            meta mark set 0xa5f3
            iifname lan01 dup to 224.0.0.251
        }
        chain mdns-saddr {
            # repeat mDNS from IoT to main
            #iifname lan-services ip saddr set 10.99.20.1
            #iifname lan-home ip saddr set 10.99.10.1
            iifname lan01 udp dport set 5354
        }
      }
*/
