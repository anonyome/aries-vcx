use std::sync::Arc;

use crate::plugins::{ledger::base_ledger::BaseLedger, wallet::base_wallet::BaseWallet, anoncreds::base_anoncreds::BaseAnonCreds, prover::base_prover::BaseProver};

pub trait Profile : std::fmt::Debug + Send + Sync {

    fn inject_ledger(self: Arc<Self>) -> Arc<dyn BaseLedger>;

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds>;

    fn inject_prover(self: Arc<Self>) -> Arc<dyn BaseProver>;

    fn inject_wallet(&self) -> Arc<dyn BaseWallet>;
}