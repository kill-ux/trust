use std::{
    collections::{hash_map::Entry, HashMap},
    io,
};

use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use trust::{Quad, TcpConnection};
use tun_tap::{Iface, Mode};

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, TcpConnection> = Default::default();
    let nic = Iface::new("tun0", Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf[..])?;
        let _eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
        if eth_proto != 0x0800 {
            // Not an IPv4 packet -- ignore
            continue;
        }
        let packet_data = &buf[4..nbytes];

        match SlicedPacket::from_ip(packet_data) {
            Ok(value) => {
                if let Some(net) = value.net {
                    match net {
                        NetSlice::Ipv4(ipv4) => {
                            let iph = ipv4.header();
                            if let Some(transport) = value.transport {
                                match transport {
                                    TransportSlice::Tcp(tcp) => {
                                        let quad = Quad::from_packet(iph, &tcp);

                                        let reply = match connections.entry(quad) {
                                            Entry::Occupied(mut occupied) => {
                                                occupied.get_mut().on_packet(iph, &tcp)
                                            }
                                            Entry::Vacant(vacant) => {
                                                if tcp.syn() {
                                                    let mut conn = TcpConnection::new(&tcp);
                                                    let response = conn.on_packet(iph, &tcp);
                                                    vacant.insert(conn);
                                                    println!("New connection! State: Listen");
                                                    response
                                                } else {
                                                    None
                                                }
                                            }
                                        };

                                        if let Some(mut packet_bytes) = reply {
                                            println!("Sending reply packet");
                                            let mut final_packet = vec![0u8, 0, 8, 0];
                                            final_packet.append(&mut packet_bytes);
                                            nic.send(&final_packet)?;
                                        }
                                    }
                                    _ => eprintln!("Non-TCP packet received"),
                                }
                            }
                        }
                        _ => {
                            eprintln!("Non-IPv4 packet received");
                        }
                    }
                }
            }
            Err(e) => eprintln!("Failed to parse packet: {:?}", e),
        }
    }
}
