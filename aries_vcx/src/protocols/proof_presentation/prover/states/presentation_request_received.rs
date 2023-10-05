use std::collections::HashMap;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
};
use messages::msg_fields::protocols::{
    present_proof::{
        present::Presentation,
        request::{RequestPresentation, RequestPresentationContent},
    },
    report_problem::ProblemReport,
};
use uuid::Uuid;

use crate::{
    common::proofs::prover::generate_indy_proof,
    errors::error::prelude::*,
    handlers::{
        proof_presentation::types::SelectedCredentials,
        util::{get_attach_as_string, Status},
    },
    protocols::proof_presentation::prover::states::{
        finished::FinishedState,
        presentation_preparation_failed::PresentationPreparationFailedState,
        presentation_prepared::PresentationPreparedState,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationRequestReceived {
    pub presentation_request: RequestPresentation,
}

impl Default for PresentationRequestReceived {
    fn default() -> Self {
        let id = Uuid::new_v4().to_string();
        let content = RequestPresentationContent::builder()
            .request_presentations_attach(Vec::new())
            .build();

        Self {
            presentation_request: RequestPresentation::builder()
                .id(id)
                .content(content)
                .build(),
        }
    }
}

impl PresentationRequestReceived {
    pub fn new(presentation_request: RequestPresentation) -> Self {
        Self {
            presentation_request,
        }
    }

    pub async fn build_presentation(
        &self,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        credentials: &SelectedCredentials,
        self_attested_attrs: &HashMap<String, String>,
    ) -> VcxResult<String> {
        let proof_req_data_json = get_attach_as_string!(
            &self
                .presentation_request
                .content
                .request_presentations_attach
        );

        generate_indy_proof(
            ledger,
            anoncreds,
            credentials,
            self_attested_attrs,
            &proof_req_data_json,
        )
        .await
    }
}

impl From<(PresentationRequestReceived, ProblemReport)> for PresentationPreparationFailedState {
    fn from((state, problem_report): (PresentationRequestReceived, ProblemReport)) -> Self {
        trace!(
            "transit state from PresentationRequestReceived to PresentationPreparationFailedState"
        );
        PresentationPreparationFailedState {
            presentation_request: state.presentation_request,
            problem_report,
        }
    }
}

impl From<(PresentationRequestReceived, Presentation)> for PresentationPreparedState {
    fn from((state, presentation): (PresentationRequestReceived, Presentation)) -> Self {
        trace!("transit state from PresentationRequestReceived to PresentationPreparedState");
        PresentationPreparedState {
            presentation_request: state.presentation_request,
            presentation,
        }
    }
}

impl From<PresentationRequestReceived> for FinishedState {
    fn from(state: PresentationRequestReceived) -> Self {
        trace!("Prover: transit state from PresentationRequestReceived to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Success,
        }
    }
}

impl From<(PresentationRequestReceived, ProblemReport)> for FinishedState {
    fn from((state, problem_report): (PresentationRequestReceived, ProblemReport)) -> Self {
        trace!("Prover: transit state from PresentationRequestReceived to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Declined(problem_report),
        }
    }
}
