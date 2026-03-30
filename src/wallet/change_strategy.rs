#[derive(Debug, Clone)]
pub enum ChangeStrategy {
    NewAddress,
    ExistingIndex(u32),
}