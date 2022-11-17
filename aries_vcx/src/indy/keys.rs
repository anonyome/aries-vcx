use vdrtools::did;
use vdrtools::future::TryFutureExt;
use vdrtools_sys::{WalletHandle};

use crate::error::prelude::*;
use crate::global::settings;
use crate::indy::utils::mocks::did_mocks::{did_mocks_enabled, DidMocks};
use crate::utils;

pub async fn create_and_store_my_did(
    wallet_handle: WalletHandle,
    seed: Option<&str>,
    method_name: Option<&str>,
) -> VcxResult<(String, String)> {
    trace!(
        "create_and_store_my_did >>> seed: {:?}, method_name: {:?}",
        seed,
        method_name
    );
    let my_did_json = json!({"seed": seed, "method_name": method_name});
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::DID.to_string(), utils::constants::VERKEY.to_string()));
    }

    let res = did::create_and_store_my_did(wallet_handle, &my_did_json.to_string())
        .await
        .map_err(VcxError::from);
    res
}

pub async fn libindy_replace_keys_start(wallet_handle: WalletHandle, did: &str) -> VcxResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("libindy_replace_keys_start >> retrieving did mock response");
        Ok(DidMocks::get_next_did_response())
    } else {
        did::replace_keys_start(wallet_handle, did, "{}")
            .map_err(VcxError::from)
            .await
    }
}

pub async fn libindy_replace_keys_apply(wallet_handle: WalletHandle, did: &str) -> VcxResult<()> {
    if did_mocks_enabled() {
        warn!("libindy_replace_keys_apply >> retrieving did mock response");
        Ok(())
    } else {
        did::replace_keys_apply(wallet_handle, did)
            .map_err(VcxError::from)
            .await
    }
}

pub async fn get_verkey_from_wallet(wallet_handle: WalletHandle, did: &str) -> VcxResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("get_verkey_from_wallet >> retrieving did mock response");
        Ok(DidMocks::get_next_did_response())
    } else {
        did::key_for_local_did(wallet_handle, did).map_err(VcxError::from).await
    }
}