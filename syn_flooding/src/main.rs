extern crate pnet;
extern crate rand;

use pnet::transport::{transport_channel, TransportProtocol};
use pnet::transport::TransportChannelType::{Layer4};
use pnet::transport::TransportProtocol::Ipv4;
use pnet::packet::ip::IpNextHeaderProtocols;

use pnet::packet::tcp::{MutableTcpPacket};
use pnet::packet::ipv4::{MutableIpv4Packet, checksum};
use pnet::packet::tcp::TcpFlags::SYN;
use pnet::packet::tcp::ipv4_checksum;
use pnet::packet::MutablePacket;
use std::net::Ipv4Addr;
use std::net::IpAddr;

// Static stuff will be changed as args at the end
static DEST_PORT: u16 = 22;
static DEST_IP: &str = "192.168.0.33";

fn main() {
    let mut tcp_buffer = [0u8; 60];
    let mut tcp_packet = MutableTcpPacket::new(&mut tcp_buffer).unwrap();

    let dest_ip = DEST_IP.parse::<Ipv4Addr>().unwrap();
    let source_ip = "192.168.0.3".parse::<Ipv4Addr>().unwrap();

    // Build TCP Packet
    tcp_packet.set_source(rand::random::<u16>());
    tcp_packet.set_destination(DEST_PORT);
    tcp_packet.set_sequence(rand::random::<u32>());
    tcp_packet.set_acknowledgement(rand::random::<u32>());
    tcp_packet.set_flags(SYN);
    tcp_packet.set_window(rand::random::<u16>());
    tcp_packet.set_checksum(ipv4_checksum(&tcp_packet.to_immutable(), &source_ip, &dest_ip));
    // Size in 32-bits word : Minimum = 5 (minium header size in bytes = 5 * 4 = 20B)
    tcp_packet.set_data_offset(5);
    // tcp_packet.set_payload(String::from("coucou").as_bytes());

    //ip_packet.set_payload(tcp_packet.packet_mut());

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
}
