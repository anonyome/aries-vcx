use std::sync::Arc;

use crate::{ledger::base_ledger::BaseLedger, wallet::base_wallet::BaseWallet};

pub trait Profile : Send + Sync {

    fn inject_ledger(self: Arc<Self>) -> Arc<dyn BaseLedger>;

    fn inject_wallet(&self) -> Arc<dyn BaseWallet>;
}