use vdrtools_sys::{PoolHandle, WalletHandle};

use crate::error::{VcxError, VcxResult};
use crate::global::settings;
use crate::indy::ledger::transactions::{
    build_cred_def_request, check_response,sign_and_submit_to_ledger
};

// consider relocating
pub async fn publish_cred_def(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    cred_def_json: &str,
) -> VcxResult<()> {
    trace!(
        "publish_cred_def >>> issuer_did: {}, cred_def_json: {}",
        issuer_did,
        cred_def_json
    );
    if settings::indy_mocks_enabled() {
        debug!("publish_cred_def >>> mocked success");
        return Ok(());
    }
    let cred_def_req = build_cred_def_request(issuer_did, cred_def_json).await?;
    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, issuer_did, &cred_def_req).await?;
    check_response(&response)
}

// consider relocating
pub async fn libindy_create_and_store_credential_def(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    schema_json: &str,
    tag: &str,
    sig_type: Option<&str>,
    config_json: &str,
) -> VcxResult<(String, String)> {
    vdrtools::anoncreds::issuer_create_and_store_credential_def(
        wallet_handle,
        issuer_did,
        schema_json,
        tag,
        sig_type,
        config_json,
    )
    .await
    .map_err(VcxError::from)
}