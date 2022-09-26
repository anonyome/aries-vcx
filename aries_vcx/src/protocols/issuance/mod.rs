use std::sync::Arc;

use crate::core::profile::profile::Profile;
use crate::error::prelude::*;
use crate::global::settings;
use crate::protocols::issuance::actions::CredentialIssuanceAction;

pub mod actions;
pub mod holder;
pub mod issuer;

pub fn verify_thread_id(thread_id: &str, message: &CredentialIssuanceAction) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!(
                "Cannot handle message {:?}: thread id does not match, expected {:?}",
                message, thread_id
            ),
        ));
    };
    Ok(())
}

pub async fn is_cred_def_revokable(profile: &Arc<dyn Profile>, cred_def_id: &str) -> VcxResult<bool> {
    let anoncreds = Arc::clone(profile).inject_anoncreds();
    let (_, cred_def_json) = anoncreds.get_cred_def_json(cred_def_id).await.map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidLedgerResponse,
            format!("Failed to obtain credential definition from ledger or cache: {}", err),
        )
    })?;
    let parsed_cred_def: serde_json::Value = serde_json::from_str(&cred_def_json).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!(
                "Failed deserialize credential definition json {}\nError: {}",
                cred_def_json, err
            ),
        )
    })?;
    Ok(!parsed_cred_def["value"]["revocation"].is_null())
}
