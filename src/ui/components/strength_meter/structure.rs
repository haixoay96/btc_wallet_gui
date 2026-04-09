/// Passphrase strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassphraseStrength {
    None,
    Weak,
    Medium,
    Strong,
    VeryStrong,
}
