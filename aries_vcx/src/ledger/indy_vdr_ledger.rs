use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::sync::Arc;

use crate::libindy::utils::ledger as libindy_ledger;
use async_trait::async_trait;
use indy_vdr::common::error::VdrError;
use indy_vdr::ledger::identifiers::{CredentialDefinitionId, SchemaId};
use indy_vdr::utils::did::DidValue;
use serde_json::Value;
use tokio::sync::oneshot;
use vdr::ledger::requests::author_agreement::{GetTxnAuthorAgreementData, TxnAuthrAgrmtAcceptanceData};
use vdr::ledger::requests::cred_def::CredentialDefinition;
use vdr::ledger::requests::schema::Schema;
use vdr::ledger::RequestBuilder;
use vdr::pool::{PoolRunner, PreparedRequest, ProtocolVersion, RequestResult};
use vdr::utils::{Qualifiable, ValidationError};

use crate::core::profile::profile::Profile;
use crate::did_doc::service_aries::AriesService;
use crate::error::VcxResult;
use crate::error::{VcxError, VcxErrorKind};
use crate::global::settings;
use crate::libindy::utils::ledger::Response;
use crate::messages::connection::did::Did;
use crate::utils::author_agreement::get_txn_author_agreement;

use indy_vdr as vdr;

use super::base_ledger::BaseLedger;

pub struct IndyVdrLedgerPool {
    runner: PoolRunner,
}

impl IndyVdrLedgerPool {
    pub fn new(runner: PoolRunner) -> Self {
        IndyVdrLedgerPool { runner }
    }
}

pub struct IndyVdrLedger {
    profile: Arc<dyn Profile>,
    pool: IndyVdrLedgerPool,
}

impl IndyVdrLedger {
    pub fn new(profile: Arc<dyn Profile>, pool: IndyVdrLedgerPool) -> Self {
        IndyVdrLedger { profile, pool }
    }

    pub fn request_builder(&self) -> VcxResult<RequestBuilder> {
        // TODO - don't use this instance of protocol version
        let v = settings::get_protocol_version();
        let version = ProtocolVersion::from_id(v as u64)?;
        Ok(RequestBuilder::new(version))
    }

    async fn _submit_request(&self, request: PreparedRequest) -> VcxResult<String> {
        // indyvdr send_request is Async via a callback.
        // Use oneshot channel to send result from callback, converting the fn to future.
        type VdrSendRequestResult =
            Result<(RequestResult<String>, Option<HashMap<String, f32, RandomState>>), VdrError>;
        let (sender, recv) = oneshot::channel::<VdrSendRequestResult>();
        self.pool.runner.send_request(
            request,
            Box::new(move |result| {
                // unable to handle a failure from `send` here
                sender.send(result).ok();
            }),
        )?;

        // todo no unwrap from recv error
        let send_req_result: VdrSendRequestResult = recv.await.unwrap();
        let (result, _) = send_req_result?;

        let reply = match result {
            RequestResult::Reply(reply) => Ok(reply),
            RequestResult::Failed(failed) => Err(failed),
        };

        Ok(reply?)
    }

    async fn _sign_and_submit_request(&self, issuer_did: &str, request: PreparedRequest) -> VcxResult<String> {
        let mut request = request;
        let to_sign = request.get_signature_input()?;

        let wallet = self.profile.inject_wallet();

        let signer_verkey = wallet.get_verkey_from_wallet(issuer_did).await?;

        let signature = self
            .profile
            .inject_wallet()
            .sign(&signer_verkey, to_sign.as_bytes())
            .await?;

        request.set_signature(&signature)?;

        self._submit_request(request).await
    }

    async fn _append_txn_author_agreement_to_request(&self, request: PreparedRequest) -> VcxResult<PreparedRequest> {
        if let Some(taa) = get_txn_author_agreement()? {
            let mut request = request;
            let acceptance = TxnAuthrAgrmtAcceptanceData {
                mechanism: taa.acceptance_mechanism_type,
                // TODO - default digest?
                taa_digest: taa.taa_digest.map_or(String::from(""), |v| v),
                time: taa.time_of_acceptance,
            };
            request.set_txn_author_agreement_acceptance(&acceptance)?;

            return Ok(request);
        } else {
            Ok(request)
        }
    }

    #[allow(dead_code)]
    async fn get_txn_author_agreement(&self) -> VcxResult<GetTxnAuthorAgreementData> {
        let request = self.build_get_txn_author_agreement_request()?;
        let response = self._submit_request(request).await?;

        let data = self.get_response_json_data_field(&response)?;

        let taa_data: GetTxnAuthorAgreementData = serde_json::from_value(data)?;

        Ok(taa_data)
    }

    fn build_get_txn_author_agreement_request(&self) -> VcxResult<PreparedRequest> {
        Ok(self
            .request_builder()?
            .build_get_txn_author_agreement_request(None, None)?)
    }

    #[allow(dead_code)]
    fn build_get_acceptance_mechanism_request(&self) -> VcxResult<PreparedRequest> {
        Ok(self
            .request_builder()?
            .build_get_acceptance_mechanisms_request(None, None, None)?)
    }

    async fn _build_get_cred_def_request(
        &self,
        submitter_did: Option<&str>,
        cred_def_id: &str,
    ) -> VcxResult<PreparedRequest> {
        let identifier = if let Some(did) = submitter_did {
            Some(DidValue::from_str(did)?)
        } else {
            None
        };
        let id = CredentialDefinitionId::from_str(cred_def_id)?;
        Ok(self
            .request_builder()?
            .build_get_cred_def_request(identifier.as_ref(), &id)?)
    }

    async fn _build_get_attr_request(
        &self,
        submitter_did: Option<&str>,
        target_did: &str,
        attribute_name: &str,
    ) -> VcxResult<PreparedRequest> {
        let identifier = if let Some(did) = submitter_did {
            Some(DidValue::from_str(did)?)
        } else {
            None
        };
        let dest = DidValue::from_str(target_did)?;
        Ok(self.request_builder()?.build_get_attrib_request(
            identifier.as_ref(),
            &dest,
            Some(attribute_name.to_string()),
            None,
            None,
        )?)
    }

    fn _build_attrib_request(
        &self,
        submitter_did: &str,
        target_did: &str,
        attrib_json_str: Option<&str>,
    ) -> VcxResult<PreparedRequest> {
        let identifier = DidValue::from_str(submitter_did)?;
        let dest = DidValue::from_str(target_did)?;
        let attrib_json = if let Some(attrib) = attrib_json_str {
            Some(serde_json::from_str::<Value>(attrib)?)
        } else {
            None
        };

        Ok(self
            .request_builder()?
            .build_attrib_request(&identifier, &dest, None, attrib_json.as_ref(), None)?)
    }

    fn get_response_json_data_field(&self, response_json: &str) -> VcxResult<Value> {
        let res: Value = serde_json::from_str(response_json)?;
        let result = (&res).try_get_index("result")?;
        Ok(result.try_get_index("data")?.to_owned())
    }
}

#[async_trait]
impl BaseLedger for IndyVdrLedger {
    async fn sign_and_submit_request(&self, issuer_did: &str, request_json: &str) -> VcxResult<String> {
        let request = PreparedRequest::from_request_json(request_json)?;

        self._sign_and_submit_request(issuer_did, request).await
    }

    async fn submit_request(&self, request_json: &str) -> VcxResult<String> {
        let request = PreparedRequest::from_request_json(request_json)?;
        self._submit_request(request).await
    }

    async fn build_schema_request(&self, submitter_did: &str, data: &str) -> VcxResult<String> {
        let identifier = DidValue::from_str(submitter_did)?;
        let schema: Schema = serde_json::from_str(data)?;
        let prepared_request = self.request_builder()?.build_schema_request(&identifier, schema)?;

        return Ok(serde_json::to_string(&prepared_request.req_json)?);
    }

    async fn build_create_credential_def_txn(
        &self,
        submitter_did: &str,
        credential_def_json: &str,
    ) -> VcxResult<String> {
        let identifier = DidValue::from_str(submitter_did)?;
        let cred_def: CredentialDefinition = serde_json::from_str(credential_def_json)?;
        let prepared_request = self.request_builder()?.build_cred_def_request(&identifier, cred_def)?;

        return Ok(serde_json::to_string(&prepared_request.req_json)?);
    }

    async fn append_txn_author_agreement_to_request(&self, request_json: &str) -> VcxResult<String> {
        let request = PreparedRequest::from_request_json(request_json)?;
        let request = self._append_txn_author_agreement_to_request(request).await?;

        return Ok(serde_json::to_string(&request.req_json)?);
    }

    async fn build_nym_request(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxResult<String> {
        let identifier = DidValue::from_str(submitter_did)?;
        let dest = DidValue::from_str(target_did)?;
        let prepared_request = self.request_builder()?.build_nym_request(
            &identifier,
            &dest,
            verkey.map(String::from),
            data.map(String::from),
            role.map(String::from),
        )?;

        return Ok(serde_json::to_string(&prepared_request.req_json)?);
    }

    async fn get_nym(&self, did: &str) -> VcxResult<String> {
        let dest = DidValue::from_str(did)?;
        let request = self.request_builder()?.build_get_nym_request(None, &dest)?;

        self._submit_request(request).await
    }

    fn parse_response(&self, response: &str) -> VcxResult<Response> {
        // sharing a libindy_ledger resource as this is a simply deserialization
        libindy_ledger::parse_response(response)
    }

    async fn get_schema(&self, submitter_did: &str, schema_id: &str) -> VcxResult<String> {
        // TODO try from cache first

        // TODO - do we need to handle someone submitting a schema request by seq number?

        let identifier = DidValue::from_str(submitter_did)?;
        let id = SchemaId::from_str(schema_id)?;

        let request = self
            .request_builder()?
            .build_get_schema_request(Some(&identifier), &id)?;

        let response = self._submit_request(request).await?;

        // TODO - process the response?
        Ok(response)
    }

    async fn build_get_cred_def_request(&self, submitter_did: Option<&str>, cred_def_id: &str) -> VcxResult<String> {
        let prepared_request = self._build_get_cred_def_request(submitter_did, cred_def_id).await?;
        return Ok(serde_json::to_string(&prepared_request.req_json)?);
    }

    async fn get_cred_def(&self, cred_def_id: &str) -> VcxResult<String> {
        // TODO try from cache first

        let request = self._build_get_cred_def_request(None, cred_def_id).await?;

        let response = self._submit_request(request).await?;

        // process the response

        let response_json: Value = serde_json::from_str(&response)?;
        let result_json = (&response_json).try_get_index("result")?;

        let schema_id = result_json.try_get_index("ref")?;
        let signature_type = result_json.try_get_index("signature_type")?;
        let tag = result_json.get("tag").map_or(json!("default"), |x| x.to_owned());
        let origin_did = result_json.try_get_index("origin")?;
        // (from ACApy) FIXME: issuer has a method to create a cred def ID
        // may need to qualify the DID
        let cred_def_id = format!(
            "{}:3:{}:{}:{}",
            origin_did.as_str_or_err()?,
            signature_type.as_str_or_err()?,
            schema_id,
            (&tag).as_str_or_err()?
        );
        let data = self.get_response_json_data_field(&response)?;

        let cred_def_value = json!({
            "ver": "1.0",
            "id": cred_def_id,
            "schemaId": schema_id.to_string(), // expected as json string, not as json int
            "type": signature_type,
            "tag": tag,
            "value": data
        });

        let cred_def_json = serde_json::to_string(&cred_def_value)?;

        Ok(cred_def_json)
    }

    async fn get_service(&self, did: &Did) -> VcxResult<AriesService> {
        let request = self._build_get_attr_request(None, &did.to_string(), "service").await?;

        let response = self._submit_request(request).await?;

        let mut data = self.get_response_json_data_field(&response)?;

        // convert `data` from JSON string to JSON Value if necessary
        if let Some(data_str) = data.as_str() {
            data = serde_json::from_str(data_str)?;
        }
        let service = (&data).try_get_index("service")?;

        serde_json::from_value(service.to_owned()).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::SerializationError,
                format!("Failed to deserialize service read from the ledger: {:?}", err),
            )
        })
    }

    async fn add_service(&self, did: &str, service: &AriesService) -> VcxResult<String> {
        let attrib_json_str = json!({ "service": service }).to_string();
        let request = self._build_attrib_request(did, did, Some(&attrib_json_str))?;
        let request = self._append_txn_author_agreement_to_request(request).await?;

        self._sign_and_submit_request(did, request).await
    }
}

impl From<VdrError> for VcxError {
    fn from(vdr_error: VdrError) -> Self {
        match vdr_error.kind() {
            // TODO - finish
            indy_vdr::common::error::VdrErrorKind::Config => {
                VcxError::from_msg(VcxErrorKind::InvalidConfiguration, vdr_error)
            }
            indy_vdr::common::error::VdrErrorKind::Connection => {
                VcxError::from_msg(VcxErrorKind::PoolLedgerConnect, vdr_error)
            }
            indy_vdr::common::error::VdrErrorKind::FileSystem(_) => {
                VcxError::from_msg(VcxErrorKind::IOError, vdr_error)
            }
            indy_vdr::common::error::VdrErrorKind::Input => {
                VcxError::from_msg(VcxErrorKind::InvalidIndyVdrInput, vdr_error)
            }
            indy_vdr::common::error::VdrErrorKind::Resource => todo!(),
            indy_vdr::common::error::VdrErrorKind::Unavailable => todo!(),
            indy_vdr::common::error::VdrErrorKind::Unexpected => todo!(),
            indy_vdr::common::error::VdrErrorKind::Incompatible => todo!(),
            indy_vdr::common::error::VdrErrorKind::PoolNoConsensus => todo!(),
            indy_vdr::common::error::VdrErrorKind::PoolRequestFailed(_) => todo!(),
            indy_vdr::common::error::VdrErrorKind::PoolTimeout => todo!(),
        }
    }
}

impl From<ValidationError> for VcxError {
    fn from(err: ValidationError) -> Self {
        VcxError::from_msg(VcxErrorKind::InvalidIndyVdrInput, err)
    }
}

trait TryGetIndex {
    type Val;
    fn try_get_index(&self, index: &str) -> Result<Self::Val, VcxError>;
}

impl<'a> TryGetIndex for &'a Value {
    type Val = &'a Value;
    fn try_get_index(&self, index: &str) -> Result<&'a Value, VcxError> {
        self.get(index).ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Could not index '{}' in IndyVDR response payload", index),
        ))
    }
}

trait AsStrOrDeserializationError {
    fn as_str_or_err(&self) -> Result<&str, VcxError>;
}

impl AsStrOrDeserializationError for &Value {
    fn as_str_or_err(&self) -> Result<&str, VcxError> {
        self.as_str().ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Could not deserialize '{}' value as string", self.to_string()),
        ))
    }
}
