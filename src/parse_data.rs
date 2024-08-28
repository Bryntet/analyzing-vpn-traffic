use crate::{BasePacket, Data, PacketDirection, TcpPacket};
use chrono::NaiveDateTime;
use itertools::Itertools;
use serde::Deserialize;
use std::fs;
use std::io::BufReader;
use rayon::prelude::*;

#[derive(Deserialize, Debug)]
pub struct RawData {
    #[serde(rename = "ip_proto")]
    ip_protocol: IpProtocol,
    #[serde(rename = "port_dst")]
    port_destination: u16,
    #[serde(rename = "port_src")]
    port_source: u16,
    #[serde(rename = "x_packets")]
    packets: Vec<Packet>,
}

#[derive(Deserialize, Debug)]
struct Packet {
    bytes: String,
    #[serde(rename = "ip_header_len")]
    ip_header_length: Option<String>,
    packets: String,
    tcp_ack_number: Option<String>,
    tcp_header_len: Option<String>,
    tcp_flags: Option<String>,
    tcp_seq_number: Option<String>,
    #[serde(with = "custom_datetime_format")]
    timestamp_start: NaiveDateTime,
    #[serde(with = "custom_datetime_format")]
    timestamp_end: NaiveDateTime,
}

mod custom_datetime_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize, Debug)]
enum IpProtocol {
    #[serde(rename = "gre")]
    Gre,
    #[serde(rename = "tcp")]
    Tcp,
    #[serde(rename = "udp")]
    Udp,
    #[serde(rename = "icmp")]
    Icmp,
}

impl From<RawData> for crate::IpProtocol {
    fn from(raw: RawData) -> Self {
        match raw.ip_protocol {
            IpProtocol::Tcp => {
                let data = Data {
                    port_destination: raw.port_destination,
                    port_source: raw.port_source,
                    packets: generate_tcp_packets(raw.packets),
                };
                crate::IpProtocol::Tcp(data)
            }
            IpProtocol::Icmp | IpProtocol::Gre | IpProtocol::Udp => {
                let data = Data {
                    port_destination: raw.port_destination,
                    port_source: raw.port_source,
                    packets: generate_packets(&raw.packets),
                };
                match raw.ip_protocol {
                    IpProtocol::Icmp => crate::IpProtocol::Icmp(data),
                    IpProtocol::Gre => crate::IpProtocol::Gre(data),
                    IpProtocol::Udp => crate::IpProtocol::Udp(data),
                    IpProtocol::Tcp => unreachable!(),
                }
            }
        }
    }
}

fn generate_packets(raw_packets: &[Packet]) -> Vec<BasePacket> {
    raw_packets
        .iter()
        .filter(|packet| packet.ip_header_length.is_some())
        .map(generate_packet)
        .collect::<Vec<_>>()
}

fn generate_packet(packet: &Packet) -> BasePacket {
    let bytes: i32 = packet.bytes.parse().unwrap();
    let packet_direction = if bytes.is_negative() {
        PacketDirection::Backward
    } else {
        PacketDirection::Forward
    };
    let ip_header_length = packet
        .ip_header_length
        .as_ref()
        .unwrap()
        .parse::<u8>()
        .unwrap();
    let packets: u8 = packet.packets.parse().unwrap();
    let duration = packet.timestamp_end - packet.timestamp_start;
    BasePacket {
        bytes: bytes.unsigned_abs(),
        direction: packet_direction,
        ip_header_length,
        packets,
        packet_duration: duration,
    }
}

fn generate_tcp_packets(raw_packets: Vec<Packet>) -> Vec<TcpPacket> {
    raw_packets
        .into_iter()
        .filter(|packet| packet.ip_header_length.is_some())
        .map(|packet| {
            let base_packet = generate_packet(&packet);
            TcpPacket {
                base: base_packet,
                tcp_header_len: packet
                    .tcp_header_len
                    .expect("TCP Header len should exist on all TCP packets")
                    .parse()
                    .unwrap(),
                tcp_flags: u8::from_str_radix(
                    packet
                        .tcp_flags
                        .expect("TCP Flags should be present on all TCP packets")
                        .as_str(),
                    2,
                )
                .unwrap(),
                tcp_acknowledgment_number: packet
                    .tcp_ack_number
                    .expect("TCP Acknowledgment number should be present on all TCP packets")
                    .parse()
                    .unwrap(),
                tcp_sequence_number: packet
                    .tcp_seq_number
                    .expect("TCP Sequence number should be present on all TCP packets")
                    .parse()
                    .unwrap(),
            }
        })
        .collect::<Vec<_>>()
}

pub fn get_data(folder: String) -> Vec<crate::IpProtocol> {
    let file = fs::File::open(folder).unwrap();
    let buf = BufReader::new(file);
    let raw_data: Vec<RawData> = serde_json::from_reader(buf).unwrap();
    raw_data
        .into_iter()
        .map(crate::IpProtocol::from)
        .collect::<Vec<_>>()
}
