use std::sync::Arc;

use async_trait::async_trait;

use crate::core::profile::indy_profile::IndySdkProfile;

use crate::indy;
use crate::xyz::primitives::revocation_registry::RevocationRegistryDefinition;
use crate::{error::VcxResult, messages::connection::did::Did, messages::did_doc::service_aries::AriesService};

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
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxResult<String> {
        indy::ledger::transactions::libindy_sign_and_submit_request(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            request_json,
        )
        .await
    }

    async fn submit_request(&self, request_json: &str) -> VcxResult<String> {
        indy::ledger::transactions::libindy_submit_request(self.profile.indy_pool_handle, request_json).await
    }

    async fn get_nym(&self, did: &str) -> VcxResult<String> {
        indy::ledger::transactions::get_nym(self.profile.indy_pool_handle, did).await
    }

    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxResult<String> {
        if let Some(submitter_did) = submitter_did {
            // with cache if possible
            indy::ledger::transactions::libindy_get_schema(
                self.profile.indy_wallet_handle,
                self.profile.indy_pool_handle,
                submitter_did,
                schema_id,
            )
            .await
        } else {
            // no cache
            indy::ledger::transactions::get_schema_json(
                self.profile.indy_wallet_handle,
                self.profile.indy_pool_handle,
                schema_id,
            )
            .await
            .map(|(_, json)| json)
        }
    }

    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxResult<String> {
        indy::ledger::transactions::libindy_get_cred_def(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            cred_def_id,
        )
        .await
    }

    async fn get_service(&self, did: &Did) -> VcxResult<AriesService> {
        indy::ledger::transactions::get_service(self.profile.indy_pool_handle, did).await
    }

    async fn add_service(&self, did: &str, service: &AriesService) -> VcxResult<String> {
        indy::ledger::transactions::add_service(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            did,
            service,
        )
        .await
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<String> {
        indy::ledger::transactions::get_rev_reg_def_json(self.profile.indy_pool_handle, rev_reg_id)
            .await
            .map(|(_, json)| json)
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)> {
        indy::ledger::transactions::get_rev_reg_delta_json(self.profile.indy_pool_handle, rev_reg_id, from, to).await
    }

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)> {
        indy::ledger::transactions::get_rev_reg(self.profile.indy_pool_handle, rev_reg_id, timestamp).await
    }

    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxResult<String> {
        indy::ledger::transactions::get_ledger_txn(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            seq_no,
            submitter_did,
        )
        .await
    }

    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxResult<()> {
        indy::primitives::credential_schema::publish_schema(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            schema_json,
            endorser_did,
        )
        .await
    }

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxResult<()> {
        indy::primitives::credential_definition::publish_cred_def(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            cred_def_json,
        )
        .await
    }

    async fn publish_rev_reg_def(
        &self,
        rev_reg_def: &RevocationRegistryDefinition,
        submitter_did: &str,
    ) -> VcxResult<()> {
        indy::primitives::revocation_registry::publish_rev_reg_def(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            rev_reg_def,
        )
        .await
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxResult<()> {
        indy::primitives::revocation_registry::publish_rev_reg_delta(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            rev_reg_id,
            rev_reg_entry_json,
        )
        .await?;

        Ok(())
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
