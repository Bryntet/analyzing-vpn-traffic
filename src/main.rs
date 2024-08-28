mod parse_data;

use std::fmt::Debug;
use crate::parse_data::{get_data, RawData};
use std::io::BufReader;
use std::fs::File;
use std::fs;
use serde::{Deserialize, Deserializer};
use chrono::{NaiveDateTime, TimeDelta};
use serde::de::Visitor;
#[derive(Debug)]
enum Encryption {
    VPN(VPN),
    NonVPN
}
#[derive(Debug)]
enum VPN {
    L2TP,
    L2TPIP,
    OpenVPN,
    PPTP,
    SSTP,
    WireGuard
}
#[derive(Debug)]
enum DataCategory {
    Mail,
    Meet,
    NonStreaming,
    SSH,
    Streaming
}
#[derive(Debug)]
enum PacketDirection {
    Forward,
    Backward
}
#[derive(Debug)]
enum IpProtocol {
    Udp(Data<BasePacket>),
    Tcp(Data<TcpPacket>),
    Gre(Data<BasePacket>),
    Icmp(Data<BasePacket>)
}


#[derive(Debug)]
struct Data<IpProtocol: Debug> {
    port_destination: u16,
    port_source: u16,
    packets: Vec<IpProtocol>
}


#[derive(Debug)]
struct BasePacket {
    bytes: u16,
    direction: PacketDirection,
    ip_header_length: u8,
    packets: u8,
    packet_duration: TimeDelta,
    
}
#[derive(Debug)]
struct TcpPacket {
    base: BasePacket,
    tcp_header_len: u16,
    tcp_flags: u8,
    tcp_acknowledgment_number: u32,
    tcp_sequence_number: u32,
}




fn main() {
    dbg!(get_data("dataset/Non VPN/ssh.json"));
}
