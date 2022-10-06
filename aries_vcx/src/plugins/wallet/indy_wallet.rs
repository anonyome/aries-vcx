use async_trait::async_trait;
use indyrs::WalletHandle;

use crate::{
    error::VcxResult,
    libindy::utils::*
};

use super::base_wallet::BaseWallet;

// #[allow(dead_code)]
#[derive(Debug)]
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
        signus::create_and_store_my_did(self.handle, seed, method_name).await
    }

    async fn get_verkey_from_wallet(&self, did: &str) -> VcxResult<String> {
        signus::get_verkey_from_wallet(self.handle, did).await
    }

    async fn add_wallet_record(&self, xtype: &str, id: &str, value: &str, tags_json: Option<&str>) -> VcxResult<()> {
        wallet::add_wallet_record(self.handle, xtype, id, value, tags_json).await
    }

    async fn get_wallet_record(&self, xtype: &str, id: &str, options_json: &str) -> VcxResult<String> {
        wallet::get_wallet_record(self.handle, xtype, id, options_json).await
    }

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxResult<()> {
        wallet::delete_wallet_record(self.handle, xtype, id).await
    }

    async fn update_wallet_record_value(&self, xtype: &str, id: &str, value: &str) -> VcxResult<()> {
        wallet::update_wallet_record_value(self.handle, xtype, id, value).await
    }

    async fn update_wallet_record_tags(&self, xtype: &str, id: &str, tags_json: &str) -> VcxResult<()> {
        wallet::update_wallet_record_tags(self.handle, xtype, id, tags_json).await
    }

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
        crypto::sign(self.handle, my_vk, msg).await
    }

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
        crypto::verify(vk, msg, signature).await
    }

    async fn pack_message(
        &self,
        sender_vk: Option<&str>,
        receiver_keys: &str,
        msg: &[u8],
    ) -> VcxResult<Vec<u8>> {
        crypto::pack_message(self.handle, sender_vk, receiver_keys, msg).await
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxResult<Vec<u8>> {
        crypto::unpack_message(self.handle, msg).await
    }
}
