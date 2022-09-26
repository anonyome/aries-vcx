use async_trait::async_trait;

use crate::error::VcxResult;

#[async_trait]
pub trait BaseProver: std::fmt::Debug + Send + Sync {
    async fn generate_indy_proof(
        &self,
        credentials: &str,
        self_attested_attrs: &str,
        proof_req_data_json: &str,
    ) -> VcxResult<String>;
}