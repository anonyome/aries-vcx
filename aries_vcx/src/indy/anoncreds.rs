use vdrtools::{anoncreds, blob_storage};

use crate::error::prelude::*;

pub(super) async fn blob_storage_open_reader(base_dir: &str) -> VcxResult<i32> {
    let tails_config = json!({"base_dir": base_dir,"uri_pattern": ""}).to_string();
    blob_storage::open_reader("default", &tails_config)
        .await
        .map_err(VcxError::from)
}

pub(super) async fn close_search_handle(search_handle: i32) -> VcxResult<()> {
    anoncreds::prover_close_credentials_search_for_proof_req(search_handle)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_to_unqualified(entity: &str) -> VcxResult<String> {
    anoncreds::to_unqualified(entity).await.map_err(VcxError::from)
}

pub async fn generate_nonce() -> VcxResult<String> {
    anoncreds::generate_nonce().await.map_err(VcxError::from)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use vdrtools_sys::WalletHandle;

    use crate::indy::ledger::transactions::get_schema_json;
    use crate::utils::constants::{SCHEMA_ID, SCHEMA_JSON};
    use crate::utils::devsetup::SetupMocks;

    #[tokio::test]
    async fn from_ledger_schema_id() {
        let _setup = SetupMocks::init();
        let (id, retrieved_schema) = get_schema_json(WalletHandle(0), 1, SCHEMA_ID).await.unwrap();
        assert_eq!(&retrieved_schema, SCHEMA_JSON);
        assert_eq!(&id, SCHEMA_ID);
    }
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use std::sync::Arc;

    use crate::core::profile::indy_profile::IndySdkProfile;
    use crate::core::profile::profile::Profile;
    use crate::indy::primitives::revocation_registry::libindy_issuer_revoke_credential;
    use crate::utils::constants::TAILS_DIR;
    use crate::utils::devsetup::SetupIndyWalletPool;
    use crate::utils::get_temp_dir_path;
    use crate::xyz::test_utils::create_and_store_credential;

    #[tokio::test]
    async fn test_issuer_revoke_credential() {
        let setup = SetupIndyWalletPool::init().await;

        let rc = libindy_issuer_revoke_credential(
            setup.wallet_handle,
            get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
            "",
            "",
        )
        .await;
        assert!(rc.is_err());

        // not the best testing strategy to construct indy profile here.
        let profile: Arc<dyn Profile> = Arc::new(IndySdkProfile::new(setup.wallet_handle, setup.pool_handle));
        let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id, _) = create_and_store_credential(
            &profile,
            &setup.institution_did,
            crate::utils::constants::DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        let rc = libindy_issuer_revoke_credential(
            setup.wallet_handle,
            get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
            &rev_reg_id,
            &cred_rev_id,
        )
        .await;

        assert!(rc.is_ok());
    }
}
