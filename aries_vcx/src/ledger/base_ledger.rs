use async_trait::async_trait;

use crate::{error::VcxResult, libindy::utils::ledger::Response, did_doc::service_aries::AriesService, messages::connection::did::Did};

#[async_trait]
pub trait BaseLedger: Send + Sync {
    async fn sign_and_submit_request(&self, issuer_did: &str, request_json: &str) -> VcxResult<String>;

    async fn submit_request(&self, request_json: &str) -> VcxResult<String>;

    async fn build_schema_request(&self, submitter_did: &str, data: &str) -> VcxResult<String>;

    async fn build_create_credential_def_txn(
        &self,
        submitter_did: &str,
        credential_def_json: &str,
    ) -> VcxResult<String>;

    async fn append_txn_author_agreement_to_request(&self, request_json: &str) -> VcxResult<String>;

    async fn build_nym_request(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxResult<String>;

    async fn get_nym(&self, did: &str) -> VcxResult<String>;

    fn parse_response(&self, response: &str) -> VcxResult<Response>;

    async fn get_schema(&self, submitter_did: &str, schema_id: &str) -> VcxResult<String>;

    async fn build_get_cred_def_request(&self, submitter_did: Option<&str>, cred_def_id: &str) -> VcxResult<String>;

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
}