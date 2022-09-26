use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    core::profile::indy_profile::IndySdkProfile, error::VcxResult, libindy::proofs::prover::prover as libindy_prover,
};

use super::base_prover::BaseProver;

#[derive(Debug)]
pub struct IndySdkProver {
    profile: Arc<IndySdkProfile>,
}

impl IndySdkProver {
    pub fn new(profile: Arc<IndySdkProfile>) -> Self {
        IndySdkProver { profile }
    }
}

#[async_trait]
impl BaseProver for IndySdkProver {
    async fn generate_indy_proof(
        &self,
        credentials: &str,
        self_attested_attrs: &str,
        proof_req_data_json: &str,
    ) -> VcxResult<String> {
        libindy_prover::generate_indy_proof(
            self.profile.indy_handle,
            credentials,
            self_attested_attrs,
            proof_req_data_json,
        )
        .await
    }
}
