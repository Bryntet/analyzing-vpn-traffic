#![allow(clippy::upper_case_acronyms, dead_code)]

mod parse_data;

use crate::parse_data::get_data;
use chrono::TimeDelta;
use std::fmt::Debug;
use std::sync::Mutex;
use psutil::process::Process;
use rayon::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

trait ToPath {
    fn path(&self) -> &'static str;
}
enum Encryption {
    VPN(VPN),
    NonVPN,
}
#[derive(EnumIter)]
enum EncryptionRepresentation {
    VPN,
    NonVPN,
}
#[allow(clippy::enum_variant_names)]
#[derive(EnumIter, Copy, Clone)]
enum VPN {
    L2TP,
    L2TPIP,
    OpenVPN,
    PPTP,
    SSTP,
    WireGuard,
}
impl ToPath for VPN {
    fn path(&self) -> &'static str {
        use VPN::*;
        match self {
            L2TP => "L2TP",
            L2TPIP => "L2TP IPsec",
            OpenVPN => "OpenVPN",
            PPTP => "PPTP",
            SSTP => "SSTP",
            WireGuard => "WireGuard",
        }
    }
}

#[derive(EnumIter)]
enum DataCategory {
    Mail,
    Meet,
    NonStreaming,
    SSH,
    Streaming,
}

impl ToPath for DataCategory {
    fn path(&self) -> &'static str {
        use DataCategory::*;
        match self {
            Mail => "mail.json",
            Meet => "meet.json",
            NonStreaming => "non_streaming.json",
            Streaming => "streaming.json",
            SSH => "ssh.json",
        }
    }
}

enum PacketDirection {
    Outgoing,
    Incoming,
}
enum IpProtocol {
    Udp(Data<BasePacket>),
    Tcp(Data<TcpPacket>),
    Gre(Data<BasePacket>),
    Icmp(Data<BasePacket>),
}

struct Data<IpProtocol: Debug> {
    port_destination: u16,
    port_source: u16,
    packets: Vec<IpProtocol>,
}

struct BasePacket {
    bytes: u32,
    direction: PacketDirection,
    ip_header_length: u8,
    packets: u8,
    packet_duration: TimeDelta,
}
struct TcpPacket {
    base: BasePacket,
    tcp_header_len: u16,
    tcp_flags: u8,
    tcp_acknowledgment_number: u32,
    tcp_sequence_number: u32,
}

#[derive(Debug)]
struct MetadataWrapper {
    encryption: Encryption,
    data_category: DataCategory,
    all_packets: Vec<IpProtocol>,
}

fn main() {
    let time = std::time::Instant::now();
    get_all_data();
    println!("Time passed: {}s", (std::time::Instant::now() - time).as_secs());
}

fn get_all_data() -> Vec<MetadataWrapper> {
    let mut all_data: Mutex<Vec<MetadataWrapper>> = Mutex::new(vec![]);
    
    EncryptionRepresentation::iter().par_bridge().for_each(|encryption_type|{
        match encryption_type {
            EncryptionRepresentation::VPN => {
                VPN::iter().par_bridge().for_each(|vpn_type|{
                    DataCategory::iter().par_bridge().for_each( |data_category|{
                        let path =
                            format!("dataset/VPN/{}/{}", vpn_type.path(), data_category.path());
                        let metadata = MetadataWrapper {
                            encryption: Encryption::VPN(vpn_type),
                            data_category,
                            all_packets: get_data(path)
                        };
                        all_data.lock().unwrap().push(metadata)
                    })
                })
            }
            EncryptionRepresentation::NonVPN => {
                DataCategory::iter().par_bridge().for_each(|data_category| {
                    let path = format!("dataset/Non VPN/{}", data_category.path());
                    let metadata = MetadataWrapper {
                        encryption: Encryption::NonVPN,
                        data_category,
                        all_packets: get_data(path),
                    };
                    all_data.lock().unwrap().push(metadata)
                })
            }
        }
    });
    all_data.into_inner().unwrap()
}


fn get_memory_usage() -> u64 {
    let current_pid = std::process::id();
    let process = Process::new(current_pid).unwrap();
    let memory_info = process.memory_info().unwrap();
    memory_info.rss()
}