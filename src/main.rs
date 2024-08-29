#![allow(clippy::upper_case_acronyms, dead_code)]

mod burn_dataset;
mod categories;
pub mod data_structure;
mod parse_data;
mod training;
mod model;

use burn::backend::{
    wgpu::{Wgpu, WgpuDevice},
    Autodiff,
};

pub fn run() {
    let device = WgpuDevice::BestAvailable;
    
    
    training::run::<Autodiff<Wgpu>>(device);
}
fn main() {
    run();
}
