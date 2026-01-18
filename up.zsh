#!/bin/zsh
PKG_NAME="trust"
cargo b --release
sudo setcap cap_net_admin=eip ./target/release/$PKG_NAME
./target/release/$PKG_NAME &
pid=$!
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set dev tun0 up
trap "kill $pid" INT EXIT
wait $pid