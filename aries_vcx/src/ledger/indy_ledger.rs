use std::sync::Arc;

use async_trait::async_trait;

use crate::core::profile::indy_profile::IndySdkProfile;

use crate::libindy::utils::ledger as libindy_ledger;
use crate::{
    did_doc::service_aries::AriesService, error::VcxResult, libindy::utils::ledger::Response,
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

    async fn build_schema_request(&self, submitter_did: &str, data: &str) -> VcxResult<String> {
        libindy_ledger::libindy_build_schema_request(submitter_did, data).await
    }

    async fn build_create_credential_def_txn(
        &self,
        submitter_did: &str,
        credential_def_json: &str,
    ) -> VcxResult<String> {
        libindy_ledger::libindy_build_create_credential_def_txn(submitter_did, credential_def_json).await
    }

    async fn append_txn_author_agreement_to_request(&self, request_json: &str) -> VcxResult<String> {
        libindy_ledger::append_txn_author_agreement_to_request(request_json).await
    }

    async fn build_nym_request(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxResult<String> {
        libindy_ledger::libindy_build_nym_request(submitter_did, target_did, verkey, data, role).await
    }

    async fn get_nym(&self, did: &str) -> VcxResult<String> {
        libindy_ledger::get_nym(did).await
    }

    fn parse_response(&self, response: &str) -> VcxResult<Response> {
        libindy_ledger::parse_response(response)
    }

    async fn get_schema(&self, submitter_did: &str, schema_id: &str) -> VcxResult<String> {
        libindy_ledger::libindy_get_schema(self.profile.indy_handle, submitter_did, schema_id).await
    }

    async fn build_get_cred_def_request(&self, submitter_did: Option<&str>, cred_def_id: &str) -> VcxResult<String> {
        libindy_ledger::libindy_build_get_cred_def_request(submitter_did, cred_def_id).await
    }

    async fn get_cred_def(&self, cred_def_id: &str) -> VcxResult<String> {
        libindy_ledger::libindy_get_cred_def(self.profile.indy_handle, cred_def_id).await
    }

    async fn get_service(&self, did: &Did) -> VcxResult<AriesService> {
        libindy_ledger::get_service(did).await
    }

    async fn add_service(&self, did: &str, service: &AriesService) -> VcxResult<String> {
        libindy_ledger::add_service(self.profile.indy_handle, did, service).await
    }
}
