const ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
use color_eyre::Result;
use pnet::datalink::interfaces;
use simple_dns::Packet;
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
};
const IFACE: [&str; 1] = ["lan01"];

fn main() -> Result<()> {
    let mut socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_port(true)?;
    let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 5353).into();
    socket.bind(&addr)?;
    socket.join_multicast_v4(&ADDR, &Ipv4Addr::new(10, 99, 10, 164))?;
    let mut buf = [0; 1024];
    loop {
        match socket.read(&mut buf) {
            Ok(_l) => {
                let packet = Packet::parse(&buf)?;
                println!("{:?}\n", packet);
            }
            Err(_) => todo!(),
        }
    }

    Ok(())
}
/*
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 5354))?;
    let interfaces = interfaces();
    let interfaces = interfaces
        .iter()
        .filter(|x| x.is_up() && !x.is_loopback() && !x.ips.is_empty());
    for i in interfaces {
        println!("{:?}", i);
    }

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
