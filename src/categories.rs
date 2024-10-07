use crate::data_structure::{BasePacket, Data, TcpPacket};
use burn::prelude::Backend;
use burn::tensor::TensorKind;
use rayon::prelude::*;
use strum_macros::{Display, EnumIter};
#[derive(Clone, Debug, PartialEq, Hash, Eq, Display)]
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
#[derive(EnumIter, Copy, Clone, Debug, PartialEq, Hash, Eq, Display)]
pub enum VPN {
    L2TP,
    L2TPIP,
    OpenVPN,
    PPTP,
    SSTP,
    WireGuard,
}
#[derive(EnumIter, Copy, Clone, Debug, Hash, PartialEq, Eq, Display)]
pub enum DataCategory {
    Mail,
    Meet,
    NonStreaming,
    SSH,
    Streaming,
}

#[derive(Clone, Debug)]
pub enum PacketDirection {
    Outgoing,
    Incoming,
}
#[derive(Clone, Debug)]
pub enum IpProtocol {
    Udp(Data<BasePacket>),
    Tcp(Data<TcpPacket>),
    Gre(Data<BasePacket>),
    Icmp(Data<BasePacket>),
}
impl<'a> From<&'a IpProtocol> for Vec<&'a BasePacket> {
    fn from(packet: &'a IpProtocol) -> Self {
        match packet {
            IpProtocol::Udp(data) | IpProtocol::Gre(data) | IpProtocol::Icmp(data) => {
                data.packets.par_iter().collect::<Vec<_>>()
            }
            IpProtocol::Tcp(data) => data
                .packets
                .par_iter()
                .map(|packet| &packet.base)
                .collect::<Vec<_>>(),
        }
    }
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
