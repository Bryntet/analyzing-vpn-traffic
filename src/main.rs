#![allow(clippy::upper_case_acronyms, dead_code)]

mod burn_dataset;
mod categories;
pub mod data_structure;
mod model;
mod parse_data;
mod training;
mod visualise;

use burn::backend::{
    wgpu::{Wgpu, WgpuDevice},
    Autodiff,
};
use crate::categories::{Encryption, VPN};

pub fn run() {
    let device = WgpuDevice::BestAvailable;

    
    training::train::<Autodiff<Wgpu>>(device);
}
fn main() {
    let hash = visualise::collect_data_specific_encryption(Encryption::VPN(VPN::L2TP));
    visualise::plot(hash).unwrap();
}
