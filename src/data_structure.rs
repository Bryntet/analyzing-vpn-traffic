use crate::categories::{
    DataCategory, Encryption, EncryptionRepresentation, IpProtocol, PacketDirection, ToPath, VPN,
};
use crate::parse_data::get_data;
use chrono::TimeDelta;
use rayon::prelude::*;
use std::sync::Mutex;
use strum::IntoEnumIterator;
pub struct Data<IpProtocol> {
    pub port_destination: u16,
    pub port_source: u16,
    pub packets: Vec<IpProtocol>,
}

pub struct BasePacket {
    pub bytes: u32,
    pub direction: PacketDirection,
    pub ip_header_length: u8,
    pub packets: u8,
    pub packet_duration: TimeDelta,
}
pub struct TcpPacket {
    pub base: BasePacket,
    pub tcp_header_len: u16,
    pub tcp_flags: u8,
    pub tcp_acknowledgment_number: u32,
    pub tcp_sequence_number: u32,
}

pub struct MetadataWrapper {
    pub encryption: Encryption,
    pub data_category: DataCategory,
    pub all_packets: Vec<IpProtocol>,
}
pub fn get_all_data() -> Vec<MetadataWrapper> {
    let all_data: Mutex<Vec<MetadataWrapper>> = Mutex::new(vec![]);

    EncryptionRepresentation::iter().par_bridge().for_each(
        |encryption_type| match encryption_type {
            EncryptionRepresentation::VPN => VPN::iter().par_bridge().for_each(|vpn_type| {
                DataCategory::iter().par_bridge().for_each(|data_category| {
                    let path = format!("dataset/VPN/{}/{}", vpn_type.path(), data_category.path());
                    let metadata = MetadataWrapper {
                        encryption: Encryption::VPN(vpn_type),
                        data_category,
                        all_packets: get_data(path),
                    };
                    all_data.lock().unwrap().push(metadata)
                })
            }),
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
        },
    );
    all_data.into_inner().unwrap()
}
