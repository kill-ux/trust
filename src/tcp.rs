use etherparse::{Ipv4HeaderSlice, TcpSlice};

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

pub struct TcpConnection {
    pub state: TcpState,
    // Next, you will add:
    // pub send_nxt: u32,
    // pub recv_nxt: u32,
}

impl TcpConnection {
    pub fn new() -> Self {
        Self {
            state: TcpState::Listen,
        }
    }

    pub fn on_packet(&mut self, iph: Ipv4HeaderSlice, tcph: TcpSlice) {
        match self.state {
            TcpState::Listen => {
                if tcph.syn() {
                    self.state = TcpState::SynRcvd;
                    // TODO: Logic to prepare a SYN-ACK
                }
            }
            TcpState::SynRcvd => {
                if tcph.ack() {
                    self.state = TcpState::Estab;
                }
            }
            TcpState::Estab => {
                // Handle data
            }
            _ => {}
        }
    }
}
