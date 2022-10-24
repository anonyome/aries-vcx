use async_trait::async_trait;

use crate::{messages::did_doc::service_aries::AriesService, error::VcxResult, messages::connection::did::Did, indy::primitives::revocation_registry::RevocationRegistryDefinition};

#[async_trait]
pub trait BaseLedger: Send + Sync {
    // multisign_request - internal
    // libindy_sign_request - internal/unused

    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxResult<String>;

    async fn submit_request(&self, request_json: &str) -> VcxResult<String>;

    // libindy_build_schema_request - internal/testing
    // libindy_build_create_credential_def_txn - internal

    // get_txn_author_agreement - todo - used in libvcx
    // append_txn_author_agreement_to_request - internal
    // libindy_build_auth_rules_request - unused
    // libindy_build_attrib_request - internal
    // libindy_build_get_auth_rule_request - unused
    // libindy_build_get_nym_request - internal
    // libindy_build_nym_request - signus

    async fn get_nym(&self, did: &str) -> VcxResult<String>;

    // get_role - internal
    // parse_response - internal

    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxResult<String>;

    // libindy_build_get_cred_def_request - internal

    async fn get_cred_def(&self, cred_def_id: &str) -> VcxResult<String>;

    async fn get_cred_def_no_cache(&self, submitter_did: Option<&str>, cred_def_id: &str) -> VcxResult<String>;

    // set_endorser - todo - used in libvcx
    // endorse_transaction - todo - used in libvcx

    // build_attrib_request - internal
    // add_attr - internal
    // get_attr - internal

    async fn get_service(&self, did: &Did) -> VcxResult<AriesService>;

    async fn add_service(&self, did: &str, service: &AriesService) -> VcxResult<String>;

    // libindy_build_revoc_reg_def_request - internal
    // libindy_build_revoc_reg_entry_request - internal

    // todo - move to helper?
    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<String>;

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)>;

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)>;

    async fn get_ledger_txn(&self, submitter_did: Option<&str>, seq_no: i32) -> VcxResult<String>;

    // build_schema_request - todo - used in libvcx

    async fn publish_schema(&self, submitter_did: &str, schema_json: &str, endorser_did: Option<String>)  -> VcxResult<()>;

    async fn publish_cred_def(&self, submitter_did: &str, cred_def_json: &str) -> VcxResult<()>;

    async fn publish_rev_reg_def(
        &self,
        submitter_did: &str,
        rev_reg_def: &RevocationRegistryDefinition,
    ) -> VcxResult<()>;

    async fn publish_rev_reg_delta(
        &self,
        submitter_did: &str,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
    ) -> VcxResult<()>;
}
