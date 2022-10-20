use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    core::profile::{indy_profile::IndySdkProfile, profile::Profile}, error::VcxResult, libindy::utils::anoncreds as libindy_anoncreds, global::settings, utils::constants::REV_REG_DELTA_JSON,
};

use super::base_anoncreds::BaseAnonCreds;

#[derive(Debug)]
pub struct IndySdkAnonCreds {
    profile: Arc<IndySdkProfile>,
}

impl IndySdkAnonCreds {
    pub fn new(profile: Arc<IndySdkProfile>) -> Self {
        IndySdkAnonCreds { profile }
    }
}

#[async_trait]
impl BaseAnonCreds for IndySdkAnonCreds {

    async fn verifier_verify_proof(
        &self,
        proof_req_json: &str,
        proof_json: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        rev_reg_defs_json: &str,
        rev_regs_json: &str,
    ) -> VcxResult<bool> {
        libindy_anoncreds::libindy_verifier_verify_proof(proof_req_json, proof_json, schemas_json, credential_defs_json, rev_reg_defs_json, rev_regs_json).await
    }

    async fn issuer_create_and_store_credential_def(
        &self,
        issuer_did: &str,
        schema_json: &str,
        tag: &str,
        sig_type: Option<&str>,
        config_json: &str,
    ) -> VcxResult<(String, String)> {
        libindy_anoncreds::libindy_issuer_create_and_store_credential_def(self.profile.indy_handle, issuer_did, schema_json, tag, sig_type, config_json).await
    }

    
    async fn issuer_create_credential_offer(
        &self, 
        cred_def_id: &str,
    ) -> VcxResult<String> {
        libindy_anoncreds::libindy_issuer_create_credential_offer(self.profile.indy_handle, cred_def_id).await
    }
    
    
    async fn issuer_create_credential(
        &self,
        cred_offer_json: &str,
        cred_req_json: &str,
        cred_values_json: &str,
        rev_reg_id: Option<String>,
        tails_file: Option<String>,
    ) -> VcxResult<(String, Option<String>, Option<String>)> {
        libindy_anoncreds::libindy_issuer_create_credential(self.profile.indy_handle, cred_offer_json, cred_req_json, cred_values_json, rev_reg_id, tails_file).await
    }

    async fn prover_create_proof(
        &self,
        proof_req_json: &str,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_create_proof(
            self.profile.indy_handle,
            proof_req_json,
            requested_credentials_json,
            master_secret_id,
            schemas_json,
            credential_defs_json,
            revoc_states_json,
        )
        .await
    }

    async fn prover_get_credentials(&self, filter_json: Option<&str>) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_get_credentials(self.profile.indy_handle, filter_json).await
    }

    async fn prover_get_credentials_for_proof_req(&self, proof_req: &str) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_get_credentials_for_proof_req(self.profile.indy_handle, proof_req).await
    }

    async fn prover_create_credential_req(
        &self,
        prover_did: &str,
        credential_offer_json: &str,
        credential_def_json: &str,
        master_secret_id: &str,
    ) -> VcxResult<(String, String)> {
        libindy_anoncreds::libindy_prover_create_credential_req(
            self.profile.indy_handle,
            prover_did,
            credential_offer_json,
            credential_def_json,
            master_secret_id,
        )
        .await
    }

    async fn prover_create_revocation_state(
        &self,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        cred_rev_id: &str,
        tails_file: &str,
    ) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_create_revocation_state(
            rev_reg_def_json,
            rev_reg_delta_json,
            cred_rev_id,
            tails_file,
        )
        .await
    }

    async fn prover_store_credential(
        &self,
        cred_id: Option<&str>,
        cred_req_meta: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_store_credential(
            self.profile.indy_handle,
            cred_id,
            cred_req_meta,
            cred_json,
            cred_def_json,
            rev_reg_def_json,
        )
        .await
    }

    async fn prover_delete_credential(&self, cred_id: &str) -> VcxResult<()> {
        libindy_anoncreds::libindy_prover_delete_credential(self.profile.indy_handle, cred_id).await
    }

    async fn prover_create_link_secret(&self, master_secret_id: &str) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_create_master_secret(self.profile.indy_handle, master_secret_id).await
    }

    async fn issuer_create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: &str,
    ) -> VcxResult<(String, String)> {
        libindy_anoncreds::libindy_issuer_create_schema(issuer_did, name, version, attrs).await
    }

    // todo - think about moving this to somewhere else as it aggregates other calls
    async fn revoke_credential_and_publish(
        &self,
        tails_file: &str,
        rev_reg_id: &str,
        cred_rev_id: &str,
    ) -> VcxResult<String> {
        libindy_anoncreds::revoke_credential(self.profile.indy_handle, tails_file, rev_reg_id, cred_rev_id).await
    }
    
    // todo - think about moving this to somewhere else as it aggregates other calls
    async fn revoke_credential_local(
        &self,
        tails_file: &str,
        rev_reg_id: &str,
        cred_rev_id: &str,
    ) -> VcxResult<()> {
        libindy_anoncreds::revoke_credential_local(self.profile.indy_handle, tails_file, rev_reg_id, cred_rev_id).await
    }
    
    // todo - think about moving this to somewhere else as it aggregates other calls
    async fn publish_local_revocations(&self, rev_reg_id: &str) -> VcxResult<String> {
        libindy_anoncreds::publish_local_revocations(self.profile.indy_handle, rev_reg_id).await
    }

    async fn generate_nonce(&self) -> VcxResult<String> {
        libindy_anoncreds::generate_nonce().await
    }
}
