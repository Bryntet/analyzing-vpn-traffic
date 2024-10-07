use crate::categories::DataCategory;
use crate::visualise::DataHash;
use itertools::Itertools;
use plotters::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::RwLock;
use strum::*;
/*pub fn plot(data: DataHash<HashMap<u32, usize>>) -> Result<(), Box<dyn std::error::Error>> {
    let max_x = RwLock::new(0u32);
    let max_y = RwLock::new(0usize);
    let min_x = RwLock::new(u32::MAX);
    let min_y = RwLock::new(usize::MAX);

    data.par_iter().for_each(|(_, categories)| {
        categories.par_iter().for_each(|(_, sizes)| {
            sizes
                .par_iter()
                .map(|(bytes, amount)| (*bytes, *amount))
                .for_each(|(bytes, amount)| {
                    if bytes > *max_x.read().unwrap() {
                        *max_x.write().unwrap() = bytes;
                    }
                    if amount > *max_y.read().unwrap() {
                        *max_y.write().unwrap() = amount
                    }
                    if bytes < *min_x.read().unwrap() {
                        *min_x.write().unwrap() = bytes
                    }
                    if amount < *min_y.read().unwrap() {
                        *min_y.write().unwrap() = amount
                    }
                })
        })
    });

    let min_x = min_x.into_inner().unwrap();
    let min_y = min_y.into_inner().unwrap();
    let max_x = max_x.into_inner().unwrap();
    let max_y = max_y.into_inner().unwrap();
    let x_range = (min_x..max_x);
    let y_range = (min_y..max_y);
    let root = BitMapBackend::new("0.png", (1920, 1080)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(1)
        .caption("Charting the packets", ("sans-serif", 30))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .build_cartesian_2d(x_range.clone(), y_range.clone())?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .y_label_formatter(&|y| format!("{y}"))
        .y_desc("Amount of packet with byte size")
        .x_desc("Bytes")
        .draw()?;

    let mut stacked_data: HashMap<u32, Vec<(DataCategory, usize)>> = HashMap::new();

    for (_, categories) in data.iter() {
        let maximum_size = *categories
            .iter()
            .flat_map(|(_, category)| category.values().max())
            .max()
            .unwrap();
        for (category, sizes) in categories.iter() {
            let total_size: usize = *sizes.values().max().unwrap();
            let mult = maximum_size as f64 / total_size as f64;
            for (&bytes, &amount) in sizes.iter() {
                stacked_data
                    .entry(bytes)
                    .or_default()
                    .push((*category, (amount as f64 * mult).round() as usize));
            }
        }
    }

    // Sort and accumulate the data for each x-value
    for (_, stack) in stacked_data.iter_mut() {
        stack.par_sort_by_key(|&(_, a)| a);
        stack.reverse();
    }

    // Draw the stacked histogram

    for category in DataCategory::iter() {
        let mut data = stacked_data
            .iter()
            .filter_map(|(&x, stack)| {
                stack
                    .iter()
                    .find(|&&(cat, _)| cat == category)
                    .map(|&(_, y)| (x, y))
            })
            .collect::<Vec<_>>();
        data.sort_by_key(|&(x, _)| x);

        let color = **color_map.get(&category).unwrap();
        chart
            .draw_series(AreaSeries::new(data, min_y, color.mix(0.2)))?
            .label(format!("{:?}", category))
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
    }

    chart.configure_series_labels().draw()?;
    root.present().expect("Failed to present chart");
    Ok(())
}*/

use eframe::egui;
use egui::Color32;
use egui_plot::{Bar, BarChart, Plot};

pub fn run_chart(data: DataHash<HashMap<u32, usize>>) -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Stacked Bar Chart Example",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, data)))),
    )
}

struct MyApp {
    data: DataHash<HashMap<u32, usize>>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>, data: DataHash<HashMap<u32, usize>>) -> Self {
        Self { data }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            Plot::new("Stacked Bar Chart")
                .allow_zoom(true)
                .allow_boxed_zoom(true)
                .legend(egui_plot::Legend::default())
                .show(ui, |plot_ui| {
                    let colors = [
                        Color32::RED,
                        Color32::GREEN,
                        Color32::BLUE,
                        Color32::GOLD,
                        Color32::ORANGE,
                        Color32::LIGHT_GRAY,
                    ];
                    let mut color_map: HashMap<DataCategory, Color32> = HashMap::new();
                    DataCategory::iter().enumerate().for_each(|(i, category)| {
                        color_map.insert(category, colors[i]);
                    });
                    self.data.iter().for_each(|(encryption, map)| {
                        map.iter().for_each(|(category, map)| {
                            let category_bar_charts = map
                                .iter()
                                .map(|(bytes, amount)| Bar::new(*bytes as f64,*amount as f64))
                                .collect_vec();
                            let chart = BarChart::new(category_bar_charts)
                                .name(category)
                                .color(*color_map.get(category).unwrap());
                            plot_ui.bar_chart(chart);
                        })
                    });
                });
        });
    }
}
