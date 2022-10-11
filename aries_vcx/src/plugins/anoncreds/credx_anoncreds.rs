use std::sync::Arc;

use crate::{
    core::profile::profile::Profile,
    error::{VcxError, VcxErrorKind, VcxResult},
    plugins::wallet::base_wallet::AsyncFnIteratorCollect,
    utils::{
        json::{AsTypeOrDeserializationError, TryGetIndex},
        uuid::uuid,
    },
};
use async_trait::async_trait;
use credx::{
    types::{
        Credential as CredxCredential, DidValue, MasterSecret, RevocationRegistryDefinition, RevocationRegistryDelta,
    },
    ursa::{bn::BigNumber, errors::UrsaCryptoError},
};
use credx::{
    types::{CredentialDefinition, CredentialOffer},
    ursa::cl::MasterSecret as UrsaMasterSecret,
};
use credx::{
    types::{CredentialRequestMetadata, PresentCredentials},
    Error as CredxError,
};
use indy_credx as credx;
use indy_vdr::utils::{Qualifiable, Validatable};
use serde_json::Value;

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

    async fn get_master_secret(&self, master_secret_id: &str) -> VcxResult<MasterSecret> {
        let wallet = self.profile.inject_wallet();

        let record = wallet
            .get_wallet_record(CATEGORY_MASTER_SECRET, master_secret_id, "{}")
            .await?;

        let record: Value = serde_json::from_str(&record)?;

        let ms_value = (&record).try_get_index("value")?;
        let ms_decimal = ms_value.as_str_or_err()?;
        let ms_bn: BigNumber = BigNumber::from_dec(ms_decimal)?;
        let ursa_ms: UrsaMasterSecret = serde_json::from_value(json!({ "ms": ms_bn }))?;

        Ok(MasterSecret { value: ursa_ms })
    }

    async fn _get_credential(&self, credential_id: &str) -> VcxResult<CredxCredential> {
        let wallet = self.profile.inject_wallet();

        let cred_record = wallet
            .get_wallet_record(CATEGORY_CREDENTIAL, credential_id, "{}")
            .await?;
        let cred_record: Value = serde_json::from_str(&cred_record)?;
        let cred_record_value = (&cred_record).try_get_index("value")?;

        let cred_json = cred_record_value.as_str_or_err()?;

        let credential: CredxCredential = serde_json::from_str(cred_json)?;

        credential.validate()?;

        Ok(credential)
    }

    async fn _get_credentials(&self, query: &str) -> VcxResult<Vec<(String, CredxCredential)>> {
        let wallet = self.profile.inject_wallet();

        let mut record_iterator = wallet.iterate_wallet_records(CATEGORY_CREDENTIAL, query, "{}").await?;
        let records = record_iterator.collect().await?;

        let id_cred_tuple_list: VcxResult<Vec<(String, CredxCredential)>> = records
            .iter()
            .map(|record| {
                let cred_record: Value = serde_json::from_str(record)?;

                let cred_record_id = (&cred_record).try_get_index("id")?.as_str_or_err()?.to_string();
                let cred_record_value = (&cred_record).try_get_index("value")?;

                let cred_json = cred_record_value.as_str_or_err()?;

                let credential: CredxCredential = serde_json::from_str(cred_json)?;

                credential.validate()?;

                Ok((cred_record_id, credential))
            })
            .collect();

        id_cred_tuple_list
    }
}

#[async_trait]
impl BaseAnonCreds for IndyCredxAnonCreds {
    /// * `requested_credentials_json`: either a credential or self-attested attribute for each requested attribute
    ///     {
    ///         "self_attested_attributes": {
    ///             "self_attested_attribute_referent": string
    ///         },
    ///         "requested_attributes": {
    ///             "requested_attribute_referent_1": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }},
    ///             "requested_attribute_referent_2": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }}
    ///         },
    ///         "requested_predicates": {
    ///             "requested_predicates_referent_1": {"cred_id": string, "timestamp": Optional<number> }},
    ///         }
    ///     }
    async fn prover_create_proof(
        &self,
        proof_req_json: &str,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxResult<String> {
        // todo revoc_states gets embedded in `credentials`

        let requested_credentials: Value = serde_json::from_str(requested_credentials_json)?;
        let requested_attributes = (&requested_credentials).try_get_index("requested_attributes")?;
        let requested_predicates = (&requested_credentials).try_get_index("requested_predicates")?;
        let self_attested_attributes = (&requested_credentials).try_get_index("self_attested_attributes")?;

        let credentials: PresentCredentials = PresentCredentials::new();

        for (attrib, val) in requested_attributes.as_object_or_err()?.iter() {
            let cred_id = val.try_get_index("cred_id")?.as_str_or_err()?;
            let timestamp = val.get("timestamp").and_then(|timestamp| timestamp.as_u64());
            let revealed = val.try_get_index("revealed")?.as_bool_or_err()?;

            let credential = CredxCredential {
                schema_id: todo!(),
                cred_def_id: todo!(),
                rev_reg_id: todo!(),
                values: todo!(),
                signature: todo!(),
                signature_correctness_proof: todo!(),
                rev_reg: todo!(),
                witness: todo!(),
            };
            // credential.
            // credentials.add_credential(, timestamp, rev_state)
        }
        // credentials.
        // credx::prover::create_presentation(pres_req, credentials, self_attested, master_secret, schemas, cred_defs)
        todo!()
    }

    async fn prover_get_credentials(&self, filter_json: Option<&str>) -> VcxResult<String> {
        // TODO - convert filter_json to query;
        let query = "{}";

        let creds = self._get_credentials("{}").await?;

        let cred_info_list: VcxResult<Vec<Value>> = creds
            .iter()
            .map(|(credential_id, cred)| _make_cred_info(credential_id, cred))
            .collect();

        let cred_info_list = cred_info_list?;

        Ok(serde_json::to_string(&cred_info_list)?)
    }

    async fn prover_get_credentials_for_proof_req(&self, proof_req: &str) -> VcxResult<String> {
        // aca-py:
        // construct "referents" from requested_attrs and requested_preds inside proof_req json
        // for each reft in referents:
        //

        let proof_req: Value = serde_json::from_str(proof_req)?;

        let requested_attributes = (&proof_req).try_get_index("requested_attributes")?;
        let requested_predicates = (&proof_req).try_get_index("requested_predicates")?;

        println!("{}", serde_json::to_string(&proof_req)?);

        todo!()
    }

    async fn prover_create_credential_req(
        &self,
        prover_did: &str,
        credential_offer_json: &str,
        credential_def_json: &str,
        master_secret_id: &str,
    ) -> VcxResult<(String, String)> {
        let prover_did = DidValue::from_str(prover_did)?;
        let cred_def: CredentialDefinition = serde_json::from_str(credential_def_json)?;
        let credential_offer: CredentialOffer = serde_json::from_str(credential_offer_json)?;
        let master_secret = self.get_master_secret(master_secret_id).await?;

        let (cred_req, cred_req_metadata) = credx::prover::create_credential_request(
            &prover_did,
            &cred_def,
            &master_secret,
            &master_secret_id,
            &credential_offer,
        )?;

        Ok((
            serde_json::to_string(&cred_req)?,
            serde_json::to_string(&cred_req_metadata)?,
        ))
    }

    async fn prover_create_revocation_state(
        &self,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        cred_rev_id: &str,
        tails_file: &str,
    ) -> VcxResult<String> {
        let tails_reader: credx::tails::TailsReader = credx::tails::TailsFileReader::new(tails_file);
        let revoc_reg_def: RevocationRegistryDefinition = serde_json::from_str(rev_reg_def_json)?;
        let rev_reg_delta: RevocationRegistryDelta = serde_json::from_str(rev_reg_delta_json)?;
        let rev_reg_idx: u32 = cred_rev_id
            .parse()
            .map_err(|e| VcxError::from_msg(VcxErrorKind::ParsingError, e))?;
        let timestamp = 100; // todo - is this ok? matches existing impl

        let rev_state = credx::prover::create_or_update_revocation_state(
            tails_reader,
            &revoc_reg_def,
            &rev_reg_delta,
            rev_reg_idx,
            timestamp,
            None,
        )?;

        Ok(serde_json::to_string(&rev_state)?)
    }

    async fn prover_store_credential(
        &self,
        cred_id: Option<&str>,
        cred_req_meta: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxResult<String> {
        let mut credential: CredxCredential = serde_json::from_str(cred_json)?;
        let cred_request_metadata: CredentialRequestMetadata = serde_json::from_str(cred_req_meta)?;
        let master_secret_id = &cred_request_metadata.master_secret_name;
        let master_secret = self.get_master_secret(master_secret_id).await?;
        let cred_def: CredentialDefinition = serde_json::from_str(cred_def_json)?;
        let rev_reg_def: Option<RevocationRegistryDefinition> = if let Some(rev_reg_def_json) = rev_reg_def_json {
            serde_json::from_str(rev_reg_def_json)?
        } else {
            None
        };

        credx::prover::process_credential(
            &mut credential,
            &cred_request_metadata,
            &master_secret,
            &cred_def,
            rev_reg_def.as_ref(),
        )?;

        credential.validate()?;

        let schema_id = &credential.schema_id;
        schema_id.validate()?;
        let (_schema_method, schema_issuer_did, schema_name, schema_version) =
            schema_id.parts().ok_or(VcxError::from_msg(
                VcxErrorKind::InvalidSchema,
                "Could not process credential.schema_id as parts.",
            ))?;

        let cred_def_id = &credential.cred_def_id;
        cred_def_id.validate()?;
        let (_cred_def_method, issuer_did, _signature_type, _schema_id, _tag) =
            cred_def_id.parts().ok_or(VcxError::from_msg(
                VcxErrorKind::InvalidSchema,
                "Could not process credential.cred_def_id as parts.",
            ))?;

        let rev_reg_id = &credential.rev_reg_id.as_ref().map(|v| &v.0);

        let mut tags = json!({
            "schema_id": schema_id.0,
            "schema_issuer_did": schema_issuer_did.0,
            "schema_name": schema_name,
            "schema_version": schema_version,
            "issuer_did": issuer_did.0,
            "cred-def_id": cred_def_id.0,
            "rev_reg_id": rev_reg_id
        });

        for (k, attr_value) in credential.values.0.iter() {
            let attr_name = _normalize_attr_name(k.to_string());
            let tag_name = format!("attr::{}::value", attr_name);
            tags[tag_name] = Value::String(attr_value.raw.to_string());
        }

        let credential_id = cred_id.map_or(uuid(), String::from);

        let record_value = serde_json::to_string(&credential)?;
        let tags_json = serde_json::to_string(&tags)?;

        self.profile
            .inject_wallet()
            .add_wallet_record(CATEGORY_CREDENTIAL, &credential_id, &record_value, Some(&tags_json))
            .await?;

        Ok(credential_id)
    }

    async fn prover_create_master_secret(&self, master_secret_id: &str) -> VcxResult<String> {
        let wallet = self.profile.inject_wallet();

        let existing_record = wallet
            .get_wallet_record(CATEGORY_MASTER_SECRET, master_secret_id, "{}")
            .await
            .ok(); // ignore error, as we only care about whether it exists or not

        if existing_record.is_some() {
            return Err(VcxError::from_msg(
                VcxErrorKind::DuplicationMasterSecret,
                format!("Master secret id: {} already exists in wallet.", master_secret_id),
            ));
        }

        // tODO - no unwrap
        let secret = credx::prover::create_master_secret()?;
        let ms_decimal = secret.value.value().unwrap().to_dec().unwrap();

        wallet
            .add_wallet_record(CATEGORY_MASTER_SECRET, master_secret_id, &ms_decimal, None)
            .await?;

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
        let rev_reg_def_json = Arc::clone(&self.profile)
            .inject_ledger()
            .get_rev_reg_def_json(rev_reg_id)
            .await?;
        Ok((rev_reg_id.to_string(), rev_reg_def_json))
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)> {
        Arc::clone(&self.profile)
            .inject_ledger()
            .get_rev_reg_delta_json(rev_reg_id, from, to)
            .await
    }

    async fn get_cred_def(&self, issuer_did: Option<&str>, cred_def_id: &str) -> VcxResult<(String, String)> {
        let cred_def_json = Arc::clone(&self.profile)
            .inject_ledger()
            .get_cred_def_no_cache(issuer_did, cred_def_id)
            .await?;

        Ok((cred_def_id.to_string(), cred_def_json))
    }
}

fn _normalize_attr_name(name: String) -> String {
    name.replace(" ", "")
}

fn _make_cred_info(credential_id: &str, cred: &CredxCredential) -> VcxResult<Value> {
    let cred_sig = serde_json::to_value(&cred.signature)?;

    let rev_info = cred_sig.get("r_credential");

    let schema_id = &cred.schema_id.0;
    let cred_def_id = &cred.cred_def_id.0;
    let rev_reg_id = cred.rev_reg_id.as_ref().map(|x| x.0.to_string());
    let cred_rev_id = rev_info.and_then(|x| x.get("i")).and_then(|i| {
        i.as_str()
            .map(|str_i| str_i.to_string())
            .or(i.as_i64().map(|int_i| int_i.to_string()))
    });

    let mut attrs = json!({});
    for (x, y) in cred.values.0.iter() {
        attrs[x] = Value::String(y.raw.to_string());
    }

    let val = json!({
        "referent": credential_id,
        "schema_id": schema_id,
        "cred_def_id": cred_def_id,
        "rev_reg_id": rev_reg_id,
        "cred_rev_id": cred_rev_id,
        "attrs": attrs
    });

    Ok(val)
}

impl From<CredxError> for VcxError {
    fn from(err: CredxError) -> Self {
        match err.kind() {
            credx::ErrorKind::Input => todo!(),
            credx::ErrorKind::IOError => todo!(),
            credx::ErrorKind::InvalidState => todo!(),
            credx::ErrorKind::Unexpected => todo!(),
            credx::ErrorKind::CredentialRevoked => todo!(),
            credx::ErrorKind::InvalidUserRevocId => todo!(),
            credx::ErrorKind::ProofRejected => todo!(),
            credx::ErrorKind::RevocationRegistryFull => todo!(),
        }
    }
}

impl From<UrsaCryptoError> for VcxError {
    fn from(err: UrsaCryptoError) -> Self {
        match err.kind() {
            credx::ursa::errors::UrsaCryptoErrorKind::InvalidState => todo!(),
            credx::ursa::errors::UrsaCryptoErrorKind::InvalidStructure => todo!(),
            credx::ursa::errors::UrsaCryptoErrorKind::InvalidParam(_) => todo!(),
            credx::ursa::errors::UrsaCryptoErrorKind::IOError => todo!(),
            credx::ursa::errors::UrsaCryptoErrorKind::ProofRejected => todo!(),
            credx::ursa::errors::UrsaCryptoErrorKind::RevocationAccumulatorIsFull => todo!(),
            credx::ursa::errors::UrsaCryptoErrorKind::InvalidRevocationAccumulatorIndex => todo!(),
            credx::ursa::errors::UrsaCryptoErrorKind::CredentialRevoked => todo!(),
        }
    }
}
