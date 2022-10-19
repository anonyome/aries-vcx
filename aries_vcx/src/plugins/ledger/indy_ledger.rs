use std::sync::Arc;

use async_trait::async_trait;

use crate::core::profile::indy_profile::IndySdkProfile;

use crate::libindy::utils::anoncreds::RevocationRegistryDefinition;
use crate::libindy::utils::ledger::{self as libindy_ledger, publish_schema};
use crate::{
    did_doc::service_aries::AriesService, error::VcxResult,
    messages::connection::did::Did,
};

use super::base_ledger::BaseLedger;

pub struct IndySdkLedger {
    profile: Arc<IndySdkProfile>,
}

impl IndySdkLedger {
    pub fn new(profile: Arc<IndySdkProfile>) -> Self {
        IndySdkLedger { profile }
    }
}

#[async_trait]
impl BaseLedger for IndySdkLedger {
    async fn sign_and_submit_request(&self, issuer_did: &str, request_json: &str) -> VcxResult<String> {
        libindy_ledger::libindy_sign_and_submit_request(self.profile.indy_handle, issuer_did, request_json).await
    }

    async fn submit_request(&self, request_json: &str) -> VcxResult<String> {
        libindy_ledger::libindy_submit_request(request_json).await
    }

    async fn get_nym(&self, did: &str) -> VcxResult<String> {
        libindy_ledger::get_nym(did).await
    }

    async fn get_schema(&self, submitter_did: Option<&str>, schema_id: &str) -> VcxResult<String> {
        // TODO - submitter did
        let submitter_did = if let Some(submitter_did) = submitter_did { submitter_did } else { "todo" };
        libindy_ledger::libindy_get_schema(self.profile.indy_handle, submitter_did, schema_id).await
    }

    async fn get_cred_def(&self, cred_def_id: &str) -> VcxResult<String> {
        libindy_ledger::libindy_get_cred_def(self.profile.indy_handle, cred_def_id).await
    }

    async fn get_cred_def_no_cache(&self, submitter_did: Option<&str>, cred_def_id: &str) -> VcxResult<String> {
        libindy_ledger::get_cred_def_no_cache(submitter_did, cred_def_id)
            .await
            .map(|(_id, json)| json)
    }

    async fn get_service(&self, did: &Did) -> VcxResult<AriesService> {
        libindy_ledger::get_service(did).await
    }

    async fn add_service(&self, did: &str, service: &AriesService) -> VcxResult<String> {
        libindy_ledger::add_service(self.profile.indy_handle, did, service).await
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<String> {
        libindy_ledger::get_rev_reg_def_json(rev_reg_id)
            .await
            .map(|(_, json)| json)
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)> {
        libindy_ledger::get_rev_reg_delta_json(rev_reg_id, from, to).await
    }

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)> {
        libindy_ledger::get_rev_reg(rev_reg_id, timestamp).await
    }

    async fn get_ledger_txn(&self, submitter_did: Option<&str>, seq_no: i32) -> VcxResult<String> {
        libindy_ledger::get_ledger_txn(self.profile.indy_handle, submitter_did, seq_no).await
    }

    async fn publish_schema(&self, schema: &str) -> VcxResult<String> {
        libindy_ledger::publish_schema(self.profile.indy_handle, schema).await
    }

    async fn publish_cred_def(&self, issuer_did: &str, cred_def_json: &str) -> VcxResult<String> {
        libindy_ledger::publish_cred_def(self.profile.indy_handle, issuer_did, cred_def_json).await
    }

    async fn publish_rev_reg_def(
        &self,
        issuer_did: &str,
        rev_reg_def: &RevocationRegistryDefinition,
    ) -> VcxResult<String> {
        libindy_ledger::publish_rev_reg_def(self.profile.indy_handle, issuer_did, rev_reg_def).await
    }

    async fn publish_rev_reg_delta(
        &self,
        issuer_did: &str,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
    ) -> VcxResult<String> {
        libindy_ledger::publish_rev_reg_delta(self.profile.indy_handle, issuer_did, rev_reg_id, rev_reg_entry_json).await
    }

    // async fn build_schema_request(&self, submitter_did: &str, data: &str) -> VcxResult<String> {
    //     libindy_ledger::libindy_build_schema_request(submitter_did, data).await
    // }

    // async fn build_create_credential_def_txn(
    //     &self,
    //     submitter_did: &str,
    //     credential_def_json: &str,
    // ) -> VcxResult<String> {
    //     libindy_ledger::libindy_build_create_credential_def_txn(submitter_did, credential_def_json).await
    // }

    // async fn append_txn_author_agreement_to_request(&self, request_json: &str) -> VcxResult<String> {
    //     libindy_ledger::append_txn_author_agreement_to_request(request_json).await
    // }

    // async fn build_nym_request(
    //     &self,
    //     submitter_did: &str,
    //     target_did: &str,
    //     verkey: Option<&str>,
    //     data: Option<&str>,
    //     role: Option<&str>,
    // ) -> VcxResult<String> {
    //     libindy_ledger::libindy_build_nym_request(submitter_did, target_did, verkey, data, role).await
    // }

    // fn parse_response(&self, response: &str) -> VcxResult<Response> {
    //     libindy_ledger::parse_response(response)
    // }
}
