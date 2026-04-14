/// Maximum number of unused addresses to scan before considering a wallet fully synced.
/// BIP-44/84 standard gap limit is 20.
pub const DEFAULT_GAP_LIMIT: u32 = 20;

/// Minimum value (in satoshis) for a valid transaction output
pub const DUST_LIMIT_SAT: u64 = 546;

/// Default fee rate (satoshis per virtual byte) used when fee estimation fails
pub const DEFAULT_AUTO_FEE_RATE_SAT_VB: f64 = 2.0;

/// Overhead vbytes for transaction headers and structure
pub const ESTIMATE_OVERHEAD_VB: u64 = 10;

/// Estimated vbytes for a P2WPKH input
pub const ESTIMATE_P2WPKH_INPUT_VB: u64 = 68;

/// Estimated vbytes for a P2WPKH output
pub const ESTIMATE_P2WPKH_OUTPUT_VB: u64 = 31;
