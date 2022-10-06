use async_trait::async_trait;

use crate::{did_doc::service_aries::AriesService, error::VcxResult, messages::connection::did::Did};

#[async_trait]
pub trait BaseLedger: Send + Sync {
    async fn get_nym(&self, did: &str) -> VcxResult<String>;

    async fn get_schema(&self, submitter_did: &str, schema_id: &str) -> VcxResult<String>;

    async fn get_cred_def(&self, cred_def_id: &str) -> VcxResult<String>;

    async fn get_cred_def_no_cache(&self, submitter_did: Option<&str>, cred_def_id: &str) -> VcxResult<String>;

    async fn get_service(&self, did: &Did) -> VcxResult<AriesService>;

    async fn add_service(&self, did: &str, service: &AriesService) -> VcxResult<String>;

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<String>;

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)>;

    async fn sign_and_submit_request(&self, issuer_did: &str, request_json: &str) -> VcxResult<String>;

    async fn submit_request(&self, request_json: &str) -> VcxResult<String>;
}
