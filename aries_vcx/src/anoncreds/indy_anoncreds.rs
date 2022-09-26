use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    core::profile::indy_profile::IndySdkProfile, error::VcxResult, libindy::utils::anoncreds as libindy_anoncreds,
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

    async fn prover_get_credentials_for_proof_req(&self, proof_req: &str) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_get_credentials_for_proof_req(self.profile.indy_handle, proof_req).await
    }

    async fn prover_create_credential_req(
        &self,
        prover_did: &str,
        credential_offer_json: &str,
        credential_def_json: &str,
    ) -> VcxResult<(String, String)> {
        libindy_anoncreds::libindy_prover_create_credential_req(self.profile.indy_handle, prover_did, credential_offer_json, credential_def_json).await
    }

    async fn prover_create_revocation_state(
        &self,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        cred_rev_id: &str,
        tails_file: &str,
    ) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_create_revocation_state(rev_reg_def_json, rev_reg_delta_json, cred_rev_id, tails_file).await
    }

    async fn prover_store_credential(
        &self,
        cred_id: Option<&str>,
        cred_req_meta: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_store_credential(self.profile.indy_handle, cred_id, cred_req_meta, cred_json, cred_def_json, rev_reg_def_json).await
    }

    async fn prover_create_master_secret(
        &self,
        master_secret_id: &str,
    ) -> VcxResult<String> {
        libindy_anoncreds::libindy_prover_create_master_secret(self.profile.indy_handle, master_secret_id).await
    }

    async fn prover_delete_credential(&self, cred_id: &str) -> VcxResult<()> {
        libindy_anoncreds::libindy_prover_delete_credential(self.profile.indy_handle, cred_id).await
    }

    async fn get_schema_json(&self, schema_id: &str) -> VcxResult<(String, String)> {
        libindy_anoncreds::get_schema_json(self.profile.indy_handle, schema_id).await
    }

    async fn get_cred_def_json(&self, cred_def_id: &str) -> VcxResult<(String, String)> {
        libindy_anoncreds::get_cred_def_json(self.profile.indy_handle, cred_def_id).await
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<(String, String)> {
        libindy_anoncreds::get_rev_reg_def_json(rev_reg_id).await
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)> {
        libindy_anoncreds::get_rev_reg_delta_json(rev_reg_id, from, to).await
    }

    async fn get_cred_def(&self, issuer_did: Option<&str>, cred_def_id: &str) -> VcxResult<(String, String)> {
        libindy_anoncreds::get_cred_def(issuer_did, cred_def_id).await
    }
}
