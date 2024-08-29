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
use burn::data::dataset::transform::{PartialDataset, ShuffledDataset};
use rayon::prelude::*;
use crate::burn_dataset::{NetworkDataset, NetworkTrafficBatcher, ShuffledData};
use crate::categories::Encryption;
use crate::data_structure::get_all_data;
use crate::model::RegressionModelConfig;

static ARTIFACT_DIR: &str = "network-analysis-model";

#[derive(Config)]
pub struct ExpConfig {
    #[config(default = 10000)]
    pub num_epochs: usize,

    #[config(default = 2)]
    pub num_workers: usize,

    #[config(default = 42)]
    pub seed: u64,

    pub optimizer: SgdConfig,

    #[config(default = 10)]
    pub input_feature_len: usize,

    #[config(default = 442)]
    pub dataset_size: usize,
}

pub fn run<B: AutodiffBackend>(device: B::Device) {
    // Config
    let optimizer = SgdConfig::new();
    let config = ExpConfig::new(optimizer);
    let model = RegressionModelConfig::new(config.input_feature_len).init(&device);
    B::seed(config.seed);

    // Define train/test datasets and dataloaders

    let data: Vec<_> = get_all_data().into_par_iter().filter(|data|data.encryption!= Encryption::NonVPN).collect();
    
    let len = data.len();

    
    // Shuffle the dataset with a defined seed such that train and test sets have no overlap
    // when splitting by indexes
    let dataset : ShuffledData = ShuffledDataset::with_seed(NetworkDataset(data.clone()), 42);

    // The dataset from HuggingFace has only train split, so we manually split the train dataset into train
    // and test in a 80-20 ratio

    let train = PartialDataset::new(dataset, 0, len * 8 / 10);
    let dataset = ShuffledDataset::with_seed(NetworkDataset(data),42);
    let test = PartialDataset::new(dataset, len * 8 / 10, len);
    



    let batcher_train = NetworkTrafficBatcher::<B>::new(device.clone());

    let batcher_test = NetworkTrafficBatcher::<B::InnerBackend>::new(device.clone());



    // Since dataset size is small, we do full batch gradient descent and set batch size equivalent to size of dataset

    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(train.len())
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(train);

    let dataloader_test = DataLoaderBuilder::new(batcher_test)
        .batch_size(test.len())
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(test);


    // Model
    let learner = LearnerBuilder::new(ARTIFACT_DIR)
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .early_stopping(MetricEarlyStoppingStrategy::new::<LossMetric<B>>(
            Aggregate::Mean,
            Direction::Lowest,
            Split::Valid,
            StoppingCondition::NoImprovementSince { n_epochs: 1 },
        ))
        .devices(vec![device.clone()])
        .num_epochs(config.num_epochs)
        .summary()
        .build(model, config.optimizer.init(), 5e-3);

    let model_trained = learner.fit(dataloader_train, dataloader_test);

    dbg!("here");
    
    config
        .save(format!("{ARTIFACT_DIR}/config.json").as_str())
        .unwrap();

    model_trained
        .save_file(
            format!("{ARTIFACT_DIR}/model"),
            &NoStdTrainingRecorder::new(),
        )
        .expect("Failed to save trained model");
}