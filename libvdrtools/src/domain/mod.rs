pub mod anoncreds;
pub mod cache;
pub mod crypto;
#[cfg(feature = "ffi_api")]
pub mod id;
pub mod ledger;
pub mod pairwise;
pub mod pool;
#[cfg(feature = "ffi_api")]
pub mod vdr;

use indy_api_types::validation::Validatable;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndyConfig {
    pub crypto_thread_pool_size: Option<usize>,
    pub collect_backtrace: Option<bool>,
    pub freshness_threshold: Option<u64>,
}

impl Validatable for IndyConfig {}