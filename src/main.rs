use std::io;

use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use tun_tap::{Iface, Mode};

fn main() -> io::Result<()> {
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
                            let src = ipv4.header().source_addr();
                            let dst = ipv4.header().destination_addr();
                            let proto = ipv4.header().protocol();

                            println!("IPv4: {} -> {} | Protocol: {:?}", src, dst, proto);

                            if let Some(transport) = value.transport {
                                match transport {
                                    TransportSlice::Tcp(tcp) => {
                                        println!("  └─ TCP Port: {} -> {}", tcp.source_port(), tcp.destination_port());
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
