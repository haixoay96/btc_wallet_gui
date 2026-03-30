use super::change_strategy::ChangeStrategy;
use super::input_source::InputSource;

#[derive(Debug, Clone)]
pub struct TxBuildOptions {
    pub broadcast: bool,
    pub input_source: InputSource,
    pub change_strategy: ChangeStrategy,
}

impl Default for TxBuildOptions {
    fn default() -> Self {
        Self {
            broadcast: false,
            input_source: InputSource::All,
            change_strategy: ChangeStrategy::NewAddress,
        }
    }
}