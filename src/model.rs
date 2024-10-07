use crate::burn_dataset::NetworkTrafficBatch;
use crate::categories::DataCategory;
use burn::nn::conv::{Conv2d, Conv2dConfig};
use burn::nn::loss::{CrossEntropyLoss, CrossEntropyLossConfig};
use burn::nn::pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig};
use burn::nn::{Dropout, DropoutConfig, LayerNorm, LayerNormConfig};
use burn::prelude::*;
use burn::tensor::activation::softmax;
use burn::train::ClassificationOutput;
use burn::{
    nn::{
        loss::{MseLoss, Reduction::Mean},
        Linear, LinearConfig, Relu,
    },
    prelude::*,
    tensor::backend::AutodiffBackend,
    train::{RegressionOutput, TrainOutput, TrainStep, ValidStep},
};
use log::{error, log};
#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    linear1: Linear<B>,
    layer_norm1: LayerNorm<B>,
    linear2: Linear<B>,
    layer_norm2: LayerNorm<B>,
    linear3: Linear<B>,
    relu: Relu,
}

impl<B: Backend> Model<B> {
    pub fn new(
        device: &B::Device,
        input_size: usize,
        hidden_size: usize,
        num_categories: usize,
    ) -> Self {
        Self {
            linear1: LinearConfig::new(input_size, hidden_size).init(device),
            layer_norm1: LayerNormConfig::new(hidden_size).init(device),
            linear2: LinearConfig::new(hidden_size, hidden_size).init(device),
            layer_norm2: LayerNormConfig::new(hidden_size).init(device),
            linear3: LinearConfig::new(hidden_size, num_categories).init(device),
            relu: Relu::new(),
        }
    }

    pub fn forward(&self, input: Tensor<B, 1>) -> Tensor<B, 1> {
        let hidden = self.linear1.forward(input);
        let hidden = self.relu.forward(hidden);
        let hidden = self.layer_norm1.forward(hidden);

        let hidden = self.linear2.forward(hidden);
        let hidden = self.relu.forward(hidden);
        let hidden = self.layer_norm2.forward(hidden);

        self.linear3.forward(hidden).squeeze(1)
    }

    pub fn forward_classification(
        &self,
        input: Tensor<B, 1>,
        targets: Tensor<B, 1, Int>,
    ) -> ClassificationOutput<B> {
        let output = self.forward(input).unsqueeze();
        let loss =
            CrossEntropyLoss::new(None, &output.device()).forward(output.clone(), targets.clone());
        ClassificationOutput::new(loss, output, targets)
    }
}

impl<B: AutodiffBackend> TrainStep<NetworkTrafficBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, batch: NetworkTrafficBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_classification(batch.inputs, batch.targets);
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<NetworkTrafficBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, batch: NetworkTrafficBatch<B>) -> ClassificationOutput<B> {
        error!("Batch inputs shape: {:?}", &batch.inputs.dims());
        self.forward_classification(batch.inputs, batch.targets)
    }
}
