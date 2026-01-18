use std::time::SystemTime;

use etherparse::{Ipv4HeaderSlice, PacketBuilder, TcpHeader, TcpSlice};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    #[default]
    Listen,
    SynRcvd,
    Estab,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAck,
    TimeWait,
    Closing,
    Closed,
}

#[derive(Debug)]
pub struct SendSequenceSpace {
    pub una: u32, // send unacknowledged
    pub nxt: u32, // send next
    pub wnd: u16, // send window
    pub up: bool, // send urgent pointer
    pub wl1: u32, // segment sequence number used for last window update
    pub wl2: u32, // segment acknowledgment number used for last window update
    pub iss: u32, // initial send sequence number
}

#[derive(Debug, Default)]
pub struct RecvSequenceSpace {
    pub nxt: u32, // receive next
    pub wnd: u16, // receive window
    pub up: bool, // receive urgent pointer
    pub irs: u32, // initial receive sequence number
}

#[derive(Debug, Default)]
pub struct TcpConnection {
    pub state: TcpState,
    pub send: SendSequenceSpace,
    pub recv: RecvSequenceSpace,
}

impl Default for SendSequenceSpace {
    fn default() -> Self {
        let iss = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        Self {
            iss,
            una: iss,
            nxt: iss + 1,
            wnd: 1024,
            up: false,
            wl1: 0,
            wl2: 0,
        }
    }
}

impl TcpConnection {
    pub fn new(tcp: &TcpSlice) -> Self {
        Self {
            state: TcpState::Listen,
            send: SendSequenceSpace::default(),
            recv: RecvSequenceSpace {
                irs: tcp.sequence_number(),
                nxt: tcp.sequence_number() + 1,
                wnd: tcp.window_size(),
                up: false,
            },
        }
    }

    pub fn on_packet(&mut self, iph: Ipv4HeaderSlice, tcp: &TcpSlice) -> Option<Vec<u8>> {
        let mut should_reply = false;
        match self.state {
            TcpState::Listen => {
                if tcp.syn() {
                    self.state = TcpState::SynRcvd;
                    should_reply = true;
                }
            }
            TcpState::SynRcvd => {
                if tcp.ack() {
                    self.state = TcpState::Estab;
                    self.send.una = tcp.acknowledgment_number();
                }
            }
            TcpState::Estab => {
                if tcp.payload().len() > 0 {
                    let payload_len = tcp.payload().len() as u32;

                    // log
                    if let Ok(msg) = std::str::from_utf8(tcp.payload()) {
                        println!("Received Data: {}", msg.trim());
                    }

                    self.recv.nxt = self.recv.nxt.wrapping_add(payload_len);

                    should_reply = true;
                }

                if tcp.fin() {
                    println!("Client requested to close connection.");
                    self.recv.nxt = self.recv.nxt.wrapping_add(1); // FIN consumes 1 sequence number
                    self.state = TcpState::CloseWait;
                    should_reply = true; // We must ACK the FIN
                }
                // Handle data
            }
            TcpState::CloseWait => {
                // Application should initiate close
                self.state = TcpState::LastAck;
                should_reply = true; // We will send FIN-ACK
            }
            _ => {}
        }

        if should_reply {
            Some(self.build_reply(&iph, tcp))
        } else {
            None
        }
    }

    pub fn build_reply(&self, iph: &Ipv4HeaderSlice, tcp: &TcpSlice) -> Vec<u8> {
        let mut tcp_header = TcpHeader::new(
            tcp.destination_port(),
            tcp.source_port(),
            self.send.iss,
            self.send.wnd,
        );

        tcp_header.acknowledgment_number = self.recv.nxt;

        match self.state {
            TcpState::SynRcvd => {
                tcp_header.syn = true;
                tcp_header.ack = true;
            }
            _ => tcp_header.ack = true,
        }

        let builder = PacketBuilder::ipv4(
            iph.destination_addr().octets(),
            iph.source_addr().octets(),
            64, // TTL
        )
        .tcp_header(tcp_header);

        let mut result = Vec::with_capacity(builder.size(0));
        builder
            .write(&mut result, &[])
            .expect("Failed to build packet");
        result
    }
}
