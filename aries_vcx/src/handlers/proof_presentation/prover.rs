use std::collections::HashMap;

use std::sync::Arc;

use agency_client::agency_client::AgencyClient;
use messages::msg_fields::protocols::present_proof::ack::AckPresentation;
use messages::msg_fields::protocols::present_proof::present::Presentation;
use messages::msg_fields::protocols::present_proof::propose::PresentationPreview;
use messages::msg_fields::protocols::present_proof::request::RequestPresentation;
use messages::AriesMessage;

use crate::core::profile::profile::Profile;
use crate::errors::error::prelude::*;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::util::{get_attach_as_string, PresentationProposalData};
use crate::protocols::proof_presentation::prover::messages::ProverMessages;
use crate::protocols::proof_presentation::prover::state_machine::{ProverSM, ProverState};
use crate::protocols::SendClosure;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Prover {
    pub prover_sm: ProverSM,
}

impl Prover {
    pub fn create(source_id: &str) -> VcxResult<Prover> {
        trace!("Prover::create >>> source_id: {}", source_id);
        Ok(Prover {
            prover_sm: ProverSM::new(source_id.to_string()),
        })
    }

    pub fn create_from_request(source_id: &str, presentation_request: RequestPresentation) -> VcxResult<Prover> {
        trace!(
            "Prover::create_from_request >>> source_id: {}, presentation_request: {:?}",
            source_id,
            presentation_request
        );
        Ok(Prover {
            prover_sm: ProverSM::from_request(presentation_request, source_id.to_string()),
        })
    }

    pub fn get_state(&self) -> ProverState {
        self.prover_sm.get_state()
    }

    pub fn presentation_status(&self) -> u32 {
        self.prover_sm.get_presentation_status()
    }

    pub async fn retrieve_credentials(&self, profile: &Arc<dyn Profile>) -> VcxResult<String> {
        trace!("Prover::retrieve_credentials >>>");
        let presentation_request = self.presentation_request_data()?;
        let anoncreds = Arc::clone(profile).inject_anoncreds();
        anoncreds
            .prover_get_credentials_for_proof_req(&presentation_request)
            .await
            .map_err(|err| err.into())
    }

    pub async fn generate_presentation(
        &mut self,
        profile: &Arc<dyn Profile>,
        credentials: String,
        self_attested_attrs: String,
    ) -> VcxResult<()> {
        trace!(
            "Prover::generate_presentation >>> credentials: {}, self_attested_attrs: {:?}",
            credentials,
            self_attested_attrs
        );
        self.prover_sm = self
            .prover_sm
            .clone()
            .generate_presentation(profile, credentials, self_attested_attrs)
            .await?;
        Ok(())
    }

    pub fn get_presentation_msg(&self) -> VcxResult<Presentation> {
        Ok(self.prover_sm.get_presentation_msg()?.to_owned())
    }

    pub fn set_presentation(&mut self, presentation: Presentation) -> VcxResult<()> {
        trace!("Prover::set_presentation >>>");
        self.prover_sm = self.prover_sm.clone().set_presentation(presentation)?;
        Ok(())
    }

    pub async fn send_proposal(
        &mut self,
        proposal_data: PresentationProposalData,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        trace!("Prover::send_proposal >>>");
        self.prover_sm = self
            .prover_sm
            .clone()
            .send_presentation_proposal(proposal_data, send_message)
            .await?;
        Ok(())
    }

    pub async fn send_presentation(&mut self, send_message: SendClosure) -> VcxResult<()> {
        trace!("Prover::send_presentation >>>");
        self.prover_sm = self.prover_sm.clone().send_presentation(send_message).await?;
        Ok(())
    }

    pub fn process_presentation_ack(&mut self, ack: AckPresentation) -> VcxResult<()> {
        trace!("Prover::process_presentation_ack >>>");
        self.prover_sm = self.prover_sm.clone().receive_presentation_ack(ack)?;
        Ok(())
    }

    pub fn progressable_by_message(&self) -> bool {
        self.prover_sm.progressable_by_message()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, AriesMessage>) -> Option<(String, AriesMessage)> {
        self.prover_sm.find_message_to_handle(messages)
    }

    pub async fn handle_message(
        &mut self,
        profile: &Arc<dyn Profile>,
        message: ProverMessages,
        send_message: Option<SendClosure>,
    ) -> VcxResult<()> {
        trace!("Prover::handle_message >>> message: {:?}", message);
        self.step(profile, message, send_message).await
    }

    pub fn presentation_request_data(&self) -> VcxResult<String> {
        Ok(get_attach_as_string!(
            &self
                .prover_sm
                .get_presentation_request()?
                .content
                .request_presentations_attach
        ))
    }

    pub fn get_proof_request_attachment(&self) -> VcxResult<String> {
        let data = get_attach_as_string!(
            &self
                .prover_sm
                .get_presentation_request()?
                .content
                .request_presentations_attach
        );

        let proof_request_data: serde_json::Value = serde_json::from_str(&data).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize {:?} into PresentationRequestData: {:?}", data, err),
            )
        })?;
        Ok(proof_request_data.to_string())
    }

    pub fn get_source_id(&self) -> String {
        self.prover_sm.source_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.prover_sm.get_thread_id()
    }

    pub async fn step(
        &mut self,
        profile: &Arc<dyn Profile>,
        message: ProverMessages,
        send_message: Option<SendClosure>,
    ) -> VcxResult<()> {
        self.prover_sm = self.prover_sm.clone().step(profile, message, send_message).await?;
        Ok(())
    }

    pub async fn decline_presentation_request(
        &mut self,
        send_message: SendClosure,
        reason: Option<String>,
        proposal: Option<String>,
    ) -> VcxResult<()> {
        trace!(
            "Prover::decline_presentation_request >>> reason: {:?}, proposal: {:?}",
            reason,
            proposal
        );
        self.prover_sm = match (reason, proposal) {
            (Some(reason), None) => {
                self.prover_sm
                    .clone()
                    .decline_presentation_request(reason, send_message)
                    .await?
            }
            (None, Some(proposal)) => {
                let presentation_preview: PresentationPreview = serde_json::from_str(&proposal).map_err(|err| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidJson,
                        format!("Cannot serialize Presentation Preview: {:?}", err),
                    )
                })?;
                self.prover_sm
                    .clone()
                    .negotiate_presentation(presentation_preview, send_message)
                    .await?
            }
            (None, None) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidOption,
                    "Either `reason` or `proposal` parameter must be specified.",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidOption,
                    "Only one of `reason` or `proposal` parameters must be specified.",
                ));
            }
        };
        Ok(())
    }

    pub async fn update_state(
        &mut self,
        profile: &Arc<dyn Profile>,
        agency_client: &AgencyClient,
        connection: &MediatedConnection,
    ) -> VcxResult<ProverState> {
        trace!("Prover::update_state >>> ");
        if !self.progressable_by_message() {
            return Ok(self.get_state());
        }
        let send_message = connection.send_message_closure(profile).await?;

        let messages = connection.get_messages(agency_client).await?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(profile, msg.into(), Some(send_message)).await?;
            connection.update_message_status(&uid, agency_client).await?;
        }
        Ok(self.get_state())
    }
}

pub mod test_utils {
    use agency_client::agency_client::AgencyClient;
    use messages::msg_fields::protocols::present_proof::PresentProof;
    use messages::AriesMessage;

    use crate::errors::error::prelude::*;
    use crate::handlers::connection::mediated_connection::MediatedConnection;

    pub async fn get_proof_request_messages(
        agency_client: &AgencyClient,
        connection: &MediatedConnection,
    ) -> VcxResult<String> {
        let presentation_requests: Vec<AriesMessage> = connection
            .get_messages(agency_client)
            .await?
            .into_iter()
            .filter_map(|(_, message)| match message {
                AriesMessage::PresentProof(PresentProof::RequestPresentation(_)) => Some(message),
                _ => None,
            })
            .collect();

        Ok(json!(presentation_requests).to_string())
    }
}

// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// mod tests {
//     use messages::msg_fields::protocols::present_proof::request::{
//         RequestPresentationContent, RequestPresentationDecorators,
//     };
//     use uuid::Uuid;

//     use crate::{common::test_utils::indy_handles_to_profile, utils::devsetup::*};
//     use aries_vcx_core::INVALID_POOL_HANDLE;

//     use super::*;

//     #[tokio::test]
//     async fn test_retrieve_credentials_fails_with_no_proof_req() {
//         SetupLibraryWallet::run(|setup| async move {
//             let profile = indy_handles_to_profile(setup.wallet_handle, INVALID_POOL_HANDLE);

//             let id = Uuid::new_v4().to_string();
//             let content = RequestPresentationContent::new(vec![]);
//             let decorators = RequestPresentationDecorators::default();

//             let proof_req = RequestPresentation::with_decorators(id, content, decorators);
//             let proof = Prover::create_from_request("1", proof_req).unwrap();
//             assert_eq!(
//                 proof.retrieve_credentials(&profile).await.unwrap_err().kind(),
//                 AriesVcxErrorKind::InvalidJson
//             );
//         })
//         .await;
//     }
// }
