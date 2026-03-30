#[derive(Debug, Clone, Copy)]
pub enum FeeMode {
    FixedSat(u64),
    Auto,
}