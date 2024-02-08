use burn::data::dataloader::batcher::Batcher;
use burn::tensor::backend::Backend;
use burn::tensor::{Data, Int, Tensor};
use serde::{Deserialize, Serialize};

use crate::ml::data;
use crate::ml::data::TrafficTraceBatcher;
use crate::trace::BinaryTrafficTrace;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BinaryTraceItem {
    pub trace: BinaryTrafficTrace,
    pub label: u8,
}

#[derive(Clone, Debug)]
pub struct BinaryBatch<B: Backend> {
    pub traces: Tensor<B, 2>,
    pub targets: Tensor<B, 1, Int>,
}

impl<B: Backend> Batcher<BinaryTraceItem, BinaryBatch<B>> for TrafficTraceBatcher<B> {
    fn batch(&self, items: Vec<BinaryTraceItem>) -> BinaryBatch<B> {
        let traces = items
            .iter()
            .map(|item| Data::<bool, 1>::from(item.trace.0.as_slice()))
            // in this step we convert all data to the backend type
            .map(|data| Tensor::<B, 1>::from_data(data.convert(), &self.device))
            .map(|tensor| {
                let size = tensor.shape().dims[0];
                tensor.reshape([1, size])
            })
            .collect();
        let traces = Tensor::cat(traces, 0).to_device(&self.device);

        BinaryBatch {
            traces,
            targets: data::build_targets(
                items.into_iter().map(|item| item.label).collect(),
                &self.device,
            ),
        }
    }
}
