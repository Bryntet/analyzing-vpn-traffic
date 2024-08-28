#![allow(clippy::upper_case_acronyms, dead_code)]

mod categories;
pub mod data_structure;
mod parse_data;

use crate::data_structure::get_all_data;

fn main() {
    let time = std::time::Instant::now();
    get_all_data();
    println!(
        "Time passed: {}s",
        (std::time::Instant::now() - time).as_secs()
    );
}
