use crate::categories::{DataCategory, Encryption, IpProtocol};
use crate::data_structure::{get_all_data, get_some_data, BasePacket, MetadataWrapper};
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

mod draw;


pub use draw::run_chart;
pub type DataHash<T> = HashMap<Encryption, HashMap<DataCategory, T>>;

#[derive(Debug, Default)]
pub struct MinAndMax {
    min: u32,
    max: u32,
}

pub fn collect_data() -> DataHash<HashMap<u32, usize>> {
    let data = get_all_data();
    collect_data_hash(data)
}
#[inline(always)]
fn collect_data_hash(data: Vec<MetadataWrapper>) -> DataHash<HashMap<u32, usize>> {
    dbg!("data loaded");
    let mut hash: HashMap<Encryption, Arc<Mutex<HashMap<DataCategory, HashMap<u32, usize>>>>> =
        HashMap::new();
    data.into_iter().for_each(|data| {
        let encryption = data.encryption;
        let category = data.data_category;
        let mut map = hash.entry(encryption).or_default().lock().unwrap();
        let maybe_vec = map.get_mut(&category);
        let amount_per_byte_size = Mutex::new(HashMap::new());

        let bytes = data
            .all_packets
            .par_iter()
            .flat_map(|packet| {
                let packet_data: Vec<&BasePacket> = packet.into();
                packet_data
                    .into_par_iter()
                    .map(|packet| packet.bytes)
            })
            .collect::<Vec<_>>();

        bytes.par_iter().for_each(|bytes| {
            *amount_per_byte_size
                .lock()
                .unwrap()
                .entry(*bytes)
                .or_default() += 1;
        });
        let amount_per_byte_size = amount_per_byte_size.into_inner().unwrap();

        if let Some(numbers) = maybe_vec {
            for (bytes, amount) in amount_per_byte_size {
                *numbers.entry(bytes).or_default() += amount;
            }
        } else {
            map.insert(category, amount_per_byte_size);
        }
    });
    hash.into_iter()
        .map(|(enc, mutex)| (enc, Arc::try_unwrap(mutex).unwrap().into_inner().unwrap()))
        .collect()
}

pub fn collect_data_specific_encryption(encryption: Encryption) -> DataHash<HashMap<u32, usize>> {
    let data = DataCategory::iter()
        .map(|category| get_some_data(encryption.clone(), category))
        .collect_vec();
    collect_data_hash(data)
}

pub fn print_all_maximum_byte_values<T: Debug>(hash: &DataHash<T>) {
    hash.iter().for_each(|(encryption, hash)| {
        hash.iter().for_each(|(category, number)| {
            let encryption = match encryption {
                Encryption::VPN(vpn) => format!("VPN: {}", vpn),
                Encryption::NonVPN => "NonVPN".to_string(),
            };
            println!("{encryption} {category} has sizes of {:#?} ", number);
        });
    })
}
