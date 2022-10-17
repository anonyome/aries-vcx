use std::thread;

use async_trait::async_trait;
use futures::executor::block_on;
use indyrs::{SearchHandle, WalletHandle};
use serde_json::Value;

use crate::{
    error::{VcxError, VcxResult},
    libindy::utils::*,
    utils::{async_fn_iterator::AsyncFnIterator, json::TryGetIndex},
};

use super::base_wallet::BaseWallet;

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

    async fn iterate_wallet_records(
        &self,
        xtype: &str,
        query: &str,
        options: &str,
    ) -> VcxResult<Box<dyn AsyncFnIterator<Item = VcxResult<String>>>> {
        let search = wallet::open_search_wallet(self.handle, xtype, query, options).await?;
        let iter = IndyWalletRecordIterator::new(self.handle, search);

        Ok(Box::new(iter))
    }

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
        crypto::sign(self.handle, my_vk, msg).await
    }

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
        crypto::verify(vk, msg, signature).await
    }

    async fn pack_message(&self, sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
        crypto::pack_message(self.handle, sender_vk, receiver_keys, msg).await
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxResult<Vec<u8>> {
        crypto::unpack_message(self.handle, msg).await
    }
}

struct IndyWalletRecordIterator {
    wallet_handle: WalletHandle,
    search_handle: SearchHandle,
}

impl IndyWalletRecordIterator {
    fn new(wallet_handle: WalletHandle, search_handle: SearchHandle) -> Self {
        IndyWalletRecordIterator {
            wallet_handle,
            search_handle,
        }
    }

    async fn fetch_next_records(&self) -> VcxResult<Option<String>> {
        let indy_res_json = wallet::fetch_next_records_wallet(self.wallet_handle, self.search_handle, 1).await?;

        let indy_res: Value = serde_json::from_str(&indy_res_json)?;

        let records = (&indy_res).try_get("records")?;

        let item: Option<VcxResult<String>> = records
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|item| Some(serde_json::to_string(item).map_err(VcxError::from)));

        item.transpose()
    }
}

#[async_trait]
impl AsyncFnIterator for IndyWalletRecordIterator {
    type Item = VcxResult<String>;

    async fn next(&mut self) -> Option<Self::Item> {
        let records = self.fetch_next_records().await;
        records.transpose()
    }
}

impl Drop for IndyWalletRecordIterator {
    fn drop(&mut self) {
        println!("DROPPING {}", self.search_handle);

        let search_handle = self.search_handle;

        thread::spawn(move || {
            block_on(async {
                wallet::close_search_wallet(search_handle).await.ok();
                println!("CLOSED {}", search_handle);
            });
        });
    }
}
