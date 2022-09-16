use async_trait::async_trait;

use crate::error::VcxResult;

#[async_trait]
pub trait BaseWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxResult<(String, String)>;

    async fn get_verkey_from_wallet(&self, did: &str) -> VcxResult<String>;

    async fn add_wallet_record(&self, xtype: &str, id: &str, value: &str, tags: Option<&str>) -> VcxResult<()>;

    async fn get_wallet_record(&self, xtype: &str, id: &str, options: &str) -> VcxResult<String>;

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxResult<()>;

    async fn update_wallet_record_value(&self, xtype: &str, id: &str, value: &str) -> VcxResult<()>;
}
