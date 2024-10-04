use crate::burn_dataset::{NetworkDataset, NetworkTrafficBatcher};
use crate::categories::{DataCategory, Encryption, VPN};
use crate::data_structure::{get_all_data, get_some_data};
use burn::data::dataset::transform::{PartialDataset, ShuffledDataset};
use burn::{
    data::{dataloader::DataLoaderBuilder, dataset::Dataset},
    optim::SgdConfig,
    prelude::*,
    record::{CompactRecorder, NoStdTrainingRecorder},
    tensor::backend::AutodiffBackend,
    train::{
        metric::store::{Aggregate, Direction, Split},
        metric::LossMetric,
        LearnerBuilder, MetricEarlyStoppingStrategy, StoppingCondition,
    },
};
use rayon::prelude::*;
use std::sync::Mutex;
use burn::train::metric::AccuracyMetric;
use rand::rngs::StdRng;
use rand::SeedableRng;
use strum::IntoEnumIterator;
use crate::model::Model;

static ARTIFACT_DIR: &str = "network-analysis-model";

#[derive(Config)]
pub struct ExpConfig {
    #[config(default = 500)]
    pub num_epochs: usize,

    #[config(default = 1)]
    pub num_workers: usize,

    #[config(default = 42)]
    pub seed: u64,

    pub optimizer: SgdConfig,

    #[config(default = 1225)]
    pub input_feature_len: usize,

    #[config(default = 1.0e-4)]
    pub learning_rate: f64,

    #[config(default = 0.9)]
    pub train_ratio: f32
}


pub fn train<B: AutodiffBackend>(device: B::Device) {
    let optimizer = SgdConfig::new();
    let config = ExpConfig::new(optimizer);
    let model = Model::new(&device,1225,1,5);

    // Set the random seed
    let data = Mutex::new(vec![]);
    DataCategory::iter().par_bridge().for_each(|category| {
        let metadata = get_some_data(Encryption::VPN(VPN::L2TP), category);
        data.lock().unwrap().push(metadata);
    });
    let mut rng = StdRng::seed_from_u64(config.seed);


    let data = NetworkDataset(data.into_inner().unwrap().into());
    let (train,learn) = data.split(config.train_ratio, &mut rng);
    // Initialize model, optimizer, and data loaders
    let optimizer = config.optimizer.init();

    let batcher_train = NetworkTrafficBatcher::<B>::new(device.clone());
    let batcher_valid = NetworkTrafficBatcher::<B::InnerBackend>::new(device.clone());

    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(64)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(train);

    let dataloader_valid = DataLoaderBuilder::new(batcher_valid)
        .batch_size(64)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(learn);

    // Set up the learner
    let learner = LearnerBuilder::new("network-analysis-model")
        .metric_train_numeric(AccuracyMetric::new())
        .metric_valid_numeric(AccuracyMetric::new())
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .devices(vec![device.clone()])
        .num_epochs(config.num_epochs)
        .build(model, optimizer, config.learning_rate);

    // Run the training
    let trained_model = learner.fit(dataloader_train, dataloader_valid);

    // Save the trained model
    trained_model
        .save_file("network-analysis-model/model".to_string(), &CompactRecorder::new())
        .unwrap();
}
