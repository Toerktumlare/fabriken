use async_trait::async_trait;

use crate::{
    channels::GlobalReceiver,
    parser::{GlobalData, Pipeline},
};

mod default;

pub use default::DefaultRuntime;

#[async_trait]
pub trait Runtime {
    async fn run(pipeline: Pipeline, global_data: GlobalData) -> GlobalReceiver;
}
