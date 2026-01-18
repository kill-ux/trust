use std::net::Ipv4Addr;

use etherparse::{Ipv4HeaderSlice, TcpSlice};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Quad {
    pub src: (Ipv4Addr, u16),
    pub dst: (Ipv4Addr, u16),
}

impl Quad {
    pub fn from_packet(iph: Ipv4HeaderSlice, tcp: &TcpSlice) -> Self {
        let src_ip = iph.source_addr();
        let dst_ip = iph.destination_addr();
        let src_port = tcp.source_port();
        let dst_port = tcp.destination_port();

        Self {
            src: (src_ip, src_port),
            dst: (dst_ip, dst_port),
        }
    }
}
