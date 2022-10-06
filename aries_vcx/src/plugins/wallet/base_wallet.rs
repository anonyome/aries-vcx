use async_trait::async_trait;

use crate::error::VcxResult;

#[async_trait]
pub trait BaseWallet: std::fmt::Debug + Send + Sync {
    // ----- DIDs

    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxResult<(String, String)>;

    async fn get_verkey_from_wallet(&self, did: &str) -> VcxResult<String>;

    // ---- records

    async fn add_wallet_record(&self, xtype: &str, id: &str, value: &str, tags_json: Option<&str>) -> VcxResult<()>;

    async fn get_wallet_record(&self, xtype: &str, id: &str, options_json: &str) -> VcxResult<String>;

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxResult<()>;

    async fn update_wallet_record_value(&self, xtype: &str, id: &str, value: &str) -> VcxResult<()>;

    async fn update_wallet_record_tags(&self, xtype: &str, id: &str, tags_json: &str) -> VcxResult<()>;

    // ---- crypto

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>>;

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool>;

    async fn pack_message(&self, sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>>;

    async fn unpack_message(&self, msg: &[u8]) -> VcxResult<Vec<u8>>;
}
