use async_trait::async_trait;

use crate::error::VcxResult;

#[async_trait]
pub trait BaseAnonCreds: std::fmt::Debug + Send + Sync {
    // SKIP (scope): libindy_verifier_verify_proof
    // SKIP (internal): libindy_create_and_store_revoc_reg
    // SKIP (scope): libindy_create_and_store_credential_def
    // SKIP (scope): libindy_issuer_create_credential_offer
    // SKIP (scoipe): libindy_issuer_create_credential

    async fn prover_create_proof(
        &self,
        proof_req_json: &str,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxResult<String>;

    async fn prover_get_credentials(&self, filter_json: Option<&str>) -> VcxResult<String>;

    async fn prover_get_credentials_for_proof_req(&self, proof_req: &str) -> VcxResult<String>;

    async fn prover_create_credential_req(
        &self,
        prover_did: &str,
        credential_offer_json: &str,
        credential_def_json: &str,
    ) -> VcxResult<(String, String)>;

    async fn prover_create_revocation_state(
        &self,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        cred_rev_id: &str,
        tails_file: &str,
    ) -> VcxResult<String>;

    // SKIP (unused): libindy_prover_update_revocation_state

    async fn prover_store_credential(
        &self,
        cred_id: Option<&str>,
        cred_req_meta: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxResult<String>;

    async fn prover_delete_credential(&self, cred_id: &str) -> VcxResult<()>;

    async fn prover_create_master_secret(
        &self,
        master_secret_id: &str,
    ) -> VcxResult<String>;

    // SKIP (internal): libindy_issuer_create_schema
    // SKIP (internal): libindy_issuer_revoke_credential
    // SKIP (internal): libindy_issuer_merge_revocation_registry_deltas
    // SKIPO (internal): libindy_build_revoc_reg_def_request
    // SIKIP (internal): libindy_build_revoc_reg_entry_request
    // SKIP (internal): libindy_build_get_revoc_reg_def_request
    // SKIP (internal): libindy_parse_get_revoc_reg_def_response
    // SKIP (internal): libindy_build_get_revoc_reg_delta_request
    // SKLIP (internal): libindy_build_get_revoc_reg_request
    // SKIP (internal;): libindy_parse_get_revoc_reg_response
    // SKIP (internal): libindy_parse_get_cred_def_response
    // SKIP (internla ): libindy_parse_get_revoc_reg_delta_response

    // SKIP (scope): create_schema
    // SKIP (scope): build_schema_request
    // SKIP (scope): publish_schema

    async fn get_schema_json(&self, schema_id: &str) -> VcxResult<(String, String)>;

    // SKIP (scope): generate_cred_def
    // SKIP (internal): build_cred_def_request
    // SKIP (scope): publish_cred_def

    // with cache
    async fn get_cred_def_json(&self, cred_def_id: &str) -> VcxResult<(String, String)>;

    // SKIP (scope): generate_rev_reg
    // SKIP (internal): build_rev_reg_request
    // SKIP (scope): publish_rev_reg_def

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<(String, String)>;

    // SKIP (internla): build_rev_reg_delta_request
    // SKIP (scope): publish_rev_reg_delta

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)>;

    // SKIP (scope): get_rev_reg

    // with no cache
    async fn get_cred_def(&self, issuer_did: Option<&str>, cred_def_id: &str) -> VcxResult<(String, String)>;

    // SKIP (internal): is_cred_def_on_ledger
    // SKIP (scope): revoke_credential
    // SKIP (scope): revoke_credential_local
    // SKIP (scope): publish_local_revocations
    // SKIP (internal): libindy_to_unqualified
    // SKIP{ (internla)}: libindy_build_get_txn_request
    // SKIP Internal: build_get_txn_request
    // SKIP (scope): get_ledger_txn
    // SKIP (tineral): _check_schema_response
    // SKIP (Internal): _check_response
    // SKIP (internla/scope): generate_nonce
    
}
