use crate::categories::{Encryption, IpProtocol, PacketDirection};
use crate::data_structure::{BasePacket, MetadataWrapper};
use burn::{
    data::{
        dataloader::batcher::Batcher,
        dataset::{
            transform::{PartialDataset, ShuffledDataset},
            Dataset,
        },
    },
    prelude::*,
};
use itertools::Itertools;
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct NetworkDataset(pub Arc<Vec<MetadataWrapper>>);
impl Dataset<MetadataWrapper> for NetworkDataset {
    fn get(&self, index: usize) -> Option<MetadataWrapper> {
        Some(self.0.get(index)?.to_owned())
    }

    fn len(&self) -> usize {
        self.0.iter().map(|data| data.all_packets.len()).sum()
    }
}

impl NetworkDataset {
    fn new(data: Vec<MetadataWrapper>) -> Self {
        Self(Arc::new(data))
    }
    pub fn split(self, train_ratio: f32, seed: &mut rand::rngs::StdRng) -> (Self, Self) {
        let total_len = self.len();
        let train_len = (total_len as f32 * train_ratio) as usize;

        // Shuffle the dataset
        let shuffled = ShuffledDataset::new(self, seed);

        let train_data: Vec<_> = (0..train_len).filter_map(|i| shuffled.get(i)).collect();
        let valid_data: Vec<_> = (train_len..total_len).filter_map(|i| shuffled.get(i)).collect();
        (
            NetworkDataset (train_data.into()),
            NetworkDataset (valid_data.into()),
        )
    }
}

#[derive(Clone, Debug)]
pub struct NetworkTrafficBatcher<B: Backend> {
    device: B::Device,
}

#[derive(Clone, Debug)]
pub struct NetworkTrafficBatch<B: Backend> {
    pub inputs: Tensor<B, 1>,
    pub targets: Tensor<B, 1, Int>,
}

impl<B: Backend> NetworkTrafficBatcher<B> {
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }

    pub fn min_max_norm<const D: usize>(&self, inp: Tensor<B, D>) -> Tensor<B, D> {
        let min = inp.clone().min_dim(0);
        let max = inp.clone().max_dim(0);
        (inp.clone() - min.clone()).div(max - min)
    }
}

impl<B: Backend> Batcher<MetadataWrapper, NetworkTrafficBatch<B>> for NetworkTrafficBatcher<B> {
    fn batch(&self, items: Vec<MetadataWrapper>) -> NetworkTrafficBatch<B> {
        let mut inputs: Vec<Tensor<B, 1>> = Vec::new();

        for item in items.iter() {
            for data in &item.all_packets {
                if let IpProtocol::Tcp(data) = data {
                    inputs.extend(
                        data.packets
                            .par_iter()
                            .map(|packet| {
                                let mut inputs = get_base_float(
                                    data.port_source,
                                    data.port_destination,
                                    &packet.base,
                                );
                                /*inputs.extend([
                                    packet.tcp_flags as f32,
                                    packet.tcp_header_len as f32,
                                    packet.tcp_acknowledgment_number as f32,
                                    packet.tcp_header_len as f32,
                                ]);*/
                                Tensor::<B, 1>::from_floats(&*inputs, &self.device)
                            })
                            .collect::<Vec<_>>(),
                    )
                } else {
                    let data = match data {
                        IpProtocol::Udp(data) | IpProtocol::Gre(data) | IpProtocol::Icmp(data) => {
                            data
                        }
                        IpProtocol::Tcp(_) => unreachable!(),
                    };
                    let things = data
                        .packets
                        .par_iter()
                        .map(|packet| {
                            Tensor::<B, 1>::from_floats(
                                &*get_base_float(data.port_source, data.port_destination, packet),
                                &self.device,
                            )
                        })
                        .collect::<Vec<_>>();
                    inputs.extend(things);
                }
            }
        }

        let inputs = Tensor::cat(inputs, 0);
        let inputs = self.min_max_norm(inputs);

        let targets = items
            .iter()
            .flat_map(|item| {
                item.all_packets.iter().map(|_| {
                    Tensor::<B, 1, Int>::from_ints(
                        [item.data_category as u8,match item.encryption {
                            Encryption::VPN(vpn) => vpn as u8 + 1,
                            Encryption::NonVPN => 0
                        }],
                        &self.device,
                    )
                }).collect::<Vec<_>>()
            })
            .collect();

        let targets = Tensor::cat(targets, 0);
        NetworkTrafficBatch { inputs, targets }
    }
}

fn get_base_float(port_source: u16, port_destination: u16, packet: &BasePacket) -> Vec<f32> {
    vec![
        port_source as f32,
        port_destination as f32,
        packet.packets as f32,
        packet.packet_duration.num_milliseconds() as f32,
        packet.ip_header_length as f32,
        match packet.direction {
            PacketDirection::Outgoing => 1.,
            PacketDirection::Incoming => 0.,
        },
        packet.bytes as f32,
    ]
}
