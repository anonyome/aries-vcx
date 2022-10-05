use std::sync::Arc;

use crate::{core::profile::profile::Profile, error::VcxResult};
use async_trait::async_trait;
use credx::types::{RevocationRegistryDefinition, RevocationRegistryDelta};
use indy_credx as credx;

use super::base_anoncreds::BaseAnonCreds;

const CATEGORY_CREDENTIAL: &str = "VCX_CREDENTIAL";
const CATEGORY_MASTER_SECRET: &str = "VCX_MASTER_CREDENTIAL";

#[derive(Debug)]
pub struct IndyCredxAnonCreds {
    profile: Arc<dyn Profile>,
}

impl IndyCredxAnonCreds {
    pub fn new(profile: Arc<dyn Profile>) -> Self {
        IndyCredxAnonCreds { profile }
    }
}

#[async_trait]
impl BaseAnonCreds for IndyCredxAnonCreds {
    async fn prover_create_proof(
        &self,
        proof_req_json: &str,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxResult<String> {
        todo!()
    }

    async fn prover_get_credentials(&self, filter_json: Option<&str>) -> VcxResult<String> {
        todo!()
    }

    async fn prover_get_credentials_for_proof_req(&self, proof_req: &str) -> VcxResult<String> {
        todo!()
    }

    async fn prover_create_credential_req(
        &self,
        prover_did: &str,
        credential_offer_json: &str,
        credential_def_json: &str,
    ) -> VcxResult<(String, String)> {
        todo!()
        // credx::
    }

    async fn prover_create_revocation_state(
        &self,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        cred_rev_id: &str,
        tails_file: &str,
    ) -> VcxResult<String> {
        let tails_reader: credx::tails::TailsReader = credx::tails::TailsFileReader::new(tails_file);
        let revoc_reg_def: RevocationRegistryDefinition = serde_json::from_str(rev_reg_def_json).unwrap();
        let rev_reg_delta: RevocationRegistryDelta = serde_json::from_str(rev_reg_delta_json).unwrap();
        let rev_reg_idx: u32 = cred_rev_id.parse().unwrap();
        let timestamp = 100; // todo - is this ok? matches existing impl

        // todo - no unwrap
        let rev_state = credx::prover::create_or_update_revocation_state(
            tails_reader,
            &revoc_reg_def,
            &rev_reg_delta,
            rev_reg_idx,
            timestamp,
            None,
        )
        .unwrap();

        Ok(serde_json::to_string(&rev_state).unwrap())
    }

    async fn prover_store_credential(
        &self,
        cred_id: Option<&str>,
        cred_req_meta: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxResult<String> {
        todo!()
    }

    async fn prover_create_master_secret(&self, master_secret_id: &str) -> VcxResult<String> {
        let _ = credx::prover::create_master_secret().unwrap();

        // todo - store in wallet
        return Ok(master_secret_id.to_string());
    }

    async fn prover_delete_credential(&self, cred_id: &str) -> VcxResult<()> {
        todo!()
    }

    async fn get_schema_json(&self, schema_id: &str) -> VcxResult<(String, String)> {
        let submitter_did = crate::utils::random::generate_random_did();
        let schema_json = Arc::clone(&self.profile)
            .inject_ledger()
            .get_schema(&submitter_did, schema_id)
            .await?;

        Ok((schema_id.to_string(), schema_json))
    }

    async fn get_cred_def_json(&self, cred_def_id: &str) -> VcxResult<(String, String)> {
        let cred_def_json = Arc::clone(&self.profile)
            .inject_ledger()
            .get_cred_def(cred_def_id)
            .await?;

            Ok((cred_def_id.to_string(), cred_def_json))
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<(String, String)> {
        let submitter_did = crate::utils::random::generate_random_did();

        let ledger = Arc::clone(&self.profile).inject_ledger();

        // ledger.rev
        // let x = ledger
        todo!()
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)> {
        todo!()
    }

    async fn get_cred_def(&self, issuer_did: Option<&str>, cred_def_id: &str) -> VcxResult<(String, String)> {
        todo!()
    }
}
