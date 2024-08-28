mod parse_data;

use std::fmt::Debug;
use crate::parse_data::{get_data, RawData};
use std::io::BufReader;
use std::fs::File;
use std::fs;
use serde::{Deserialize, Deserializer};
use chrono::{NaiveDateTime, TimeDelta};
use serde::de::Visitor;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;


trait ToPath {
    fn path(&self) -> &'static str;
}
#[derive(Debug)]
enum Encryption {
    VPN(VPN),
    NonVPN
}
#[derive(EnumIter)]
enum EncryptionRepresentation {
    VPN,
    NonVPN
}




#[derive(Debug, EnumIter, Copy, Clone)]
enum VPN {
    L2TP,
    L2TPIP,
    OpenVPN,
    PPTP,
    SSTP,
    WireGuard
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


#[derive(Debug, EnumIter)]
enum DataCategory {
    Mail,
    Meet,
    NonStreaming,
    SSH,
    Streaming
}

impl ToPath for DataCategory {
    fn path(&self) -> &'static str {
        use DataCategory::*;
        match self {
            Mail => "mail.json",
            Meet => "meet.json",
            NonStreaming => "non_streaming.json",
            Streaming => "streaming.json",
            SSH => "ssh.json"
        }
    }
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

#[derive(Debug)]
struct MetadataWrapper {
    encryption: Encryption,
    data_category: DataCategory,
    all_packets: Vec<IpProtocol>
}





fn main() {
   get_all_data(); 
}


fn get_all_data() -> Vec<MetadataWrapper> {
    let mut all_data: Vec<MetadataWrapper> = vec![];
    for encryption_type in EncryptionRepresentation::iter() {
        match encryption_type {
            EncryptionRepresentation::VPN => {
                for vpn_type in VPN::iter() {
                    for data_category in DataCategory::iter() {
                        let path = format!("dataset/VPN/{}/{}",vpn_type.path(),data_category.path());
                        let data = get_data(path);
                        all_data.push(MetadataWrapper {
                            encryption: Encryption::VPN(vpn_type),
                            data_category,
                            all_packets: data
                        })
                    }
                }
            }
            EncryptionRepresentation::NonVPN => {
                for data_category in DataCategory::iter() {
                    let path = format!("dataset/Non VPN/{}",data_category.path());
                    let data = get_data(path);
                    all_data.push(MetadataWrapper {
                        encryption: Encryption::NonVPN,
                        data_category,
                        all_packets: data
                    })
                }
            }
        }
    }
    all_data
}