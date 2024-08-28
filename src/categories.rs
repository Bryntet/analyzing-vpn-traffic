use crate::data_structure::{BasePacket, Data, TcpPacket};
use strum_macros::EnumIter;

pub enum Encryption {
    VPN(VPN),
    NonVPN,
}
#[derive(EnumIter)]
pub enum EncryptionRepresentation {
    VPN,
    NonVPN,
}
#[allow(clippy::enum_variant_names)]
#[derive(EnumIter, Copy, Clone)]
pub enum VPN {
    L2TP,
    L2TPIP,
    OpenVPN,
    PPTP,
    SSTP,
    WireGuard,
}
#[derive(EnumIter)]
pub enum DataCategory {
    Mail,
    Meet,
    NonStreaming,
    SSH,
    Streaming,
}
pub enum PacketDirection {
    Outgoing,
    Incoming,
}
pub enum IpProtocol {
    Udp(Data<BasePacket>),
    Tcp(Data<TcpPacket>),
    Gre(Data<BasePacket>),
    Icmp(Data<BasePacket>),
}
pub trait ToPath {
    fn path(&self) -> &'static str;
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
