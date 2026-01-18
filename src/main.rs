use std::io;

use tun_tap::{Iface, Mode};

fn main() -> io::Result<()> {
    let nic = Iface::new("tun0", Mode::Tun)?;

    let mut buf = [0u8; 1504];

    loop {
        let nbytes = nic.recv(&mut buf[..])?;
        println!("received {} bytes: {:x?}", nbytes, &buf[..nbytes]);
    }
}
