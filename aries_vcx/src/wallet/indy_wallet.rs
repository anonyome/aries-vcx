use async_trait::async_trait;
use indyrs::WalletHandle;

use crate::{
    error::VcxResult,
    libindy::utils::{
        signus::{create_and_store_my_did, get_verkey_from_wallet},
        wallet::{add_wallet_record, get_wallet_record},
    },
};

use super::base_wallet::BaseWallet;

#[allow(dead_code)]
pub struct IndySdkWallet {
    handle: WalletHandle,
}

impl IndySdkWallet {
    pub fn new(handle: WalletHandle) -> Self {
        IndySdkWallet { handle }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl BaseWallet for IndySdkWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxResult<(String, String)> {
        create_and_store_my_did(self.handle, seed, method_name).await
    }

    async fn get_verkey_from_wallet(&self, did: &str) -> VcxResult<String> {
        get_verkey_from_wallet(self.handle, did).await
    }

    async fn add_wallet_record(&self, xtype: &str, id: &str, value: &str, tags: Option<&str>) -> VcxResult<()> {
        add_wallet_record(self.handle, xtype, id, value, tags).await
    }

    async fn get_wallet_record(&self, xtype: &str, id: &str, options: &str) -> VcxResult<String> {
        get_wallet_record(self.handle, xtype, id, options).await
    }

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxResult<()> {
        todo!()
    }

    async fn update_wallet_record_value(&self, xtype: &str, id: &str, value: &str) -> VcxResult<()> {
        todo!()
    }
}
