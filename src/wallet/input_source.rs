#[derive(Debug, Clone)]
pub enum InputSource {
    All,
    AddressIndexes(Vec<u32>),
}