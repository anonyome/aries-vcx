use std::sync::Arc;

use crate::{
    error::{VcxError, VcxErrorKind, VcxResult},
    utils::async_fn_iterator::AsyncFnIterator,
};

use super::base_wallet::BaseWallet;
use agency_client::{
    error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    wallet::base_agency_client_wallet::BaseAgencyClientWallet,
};
use async_trait::async_trait;

#[derive(Debug)]
pub(crate) struct AgencyClientWallet {
    inner: Arc<dyn BaseAgencyClientWallet>,
}

#[allow(unused_variables)]
#[async_trait]
impl BaseWallet for AgencyClientWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxResult<(String, String)> {
        Err(unimplemented_agency_client_wallet_method("create_and_store_my_did"))
    }

    async fn get_verkey_from_wallet(&self, did: &str) -> VcxResult<String> {
        Err(unimplemented_agency_client_wallet_method("get_verkey_from_wallet"))
    }

    async fn add_wallet_record(&self, xtype: &str, id: &str, value: &str, tags_json: Option<&str>) -> VcxResult<()> {
        Err(unimplemented_agency_client_wallet_method("add_wallet_record"))
    }

    async fn get_wallet_record(&self, xtype: &str, id: &str, options_json: &str) -> VcxResult<String> {
        Err(unimplemented_agency_client_wallet_method("get_wallet_record"))
    }

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxResult<()> {
        Err(unimplemented_agency_client_wallet_method("delete_wallet_record"))
    }

    async fn update_wallet_record_value(&self, xtype: &str, id: &str, value: &str) -> VcxResult<()> {
        Err(unimplemented_agency_client_wallet_method("update_wallet_record_value"))
    }

    async fn update_wallet_record_tags(&self, xtype: &str, id: &str, tags_json: &str) -> VcxResult<()> {
        Err(unimplemented_agency_client_wallet_method("update_wallet_record_tags"))
    }

    async fn iterate_wallet_records(
        &self,
        xtype: &str,
        query: &str,
        options: &str,
    ) -> VcxResult<Box<dyn AsyncFnIterator<Item = VcxResult<String>>>> {
        Err(unimplemented_agency_client_wallet_method("iterate_wallet_records"))
    }

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
        Err(unimplemented_agency_client_wallet_method("sign"))
    }

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
        Err(unimplemented_agency_client_wallet_method("verify"))
    }

    async fn pack_message(&self, sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
        Ok(self.inner.pack_message(sender_vk, receiver_keys, msg).await?)
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxResult<Vec<u8>> {
        Ok(self.inner.unpack_message(msg).await?)
    }
}

pub(crate) trait ToBaseWallet {
    fn to_base_wallet(&self) -> Arc<dyn BaseWallet>;
}

impl ToBaseWallet for Arc<dyn BaseAgencyClientWallet> {
    fn to_base_wallet(&self) -> Arc<dyn BaseWallet> {
        Arc::new(AgencyClientWallet {
            inner: Arc::clone(self),
        })
    }
}

fn unimplemented_agency_client_wallet_method(method_name: &str) -> VcxError {
    VcxError::from_msg(
        VcxErrorKind::UnimplementedFeature,
        format!("AgencyClientWallet::{} is not intended to be used.", method_name),
    )
}

// --------------------------------------------

#[derive(Debug)]
pub(crate) struct BaseWalletAgencyClientWallet {
    inner: Arc<dyn BaseWallet>,
}

#[async_trait]
impl BaseAgencyClientWallet for BaseWalletAgencyClientWallet {
    async fn pack_message(
        &self,
        sender_vk: Option<&str>,
        receiver_keys: &str,
        msg: &[u8],
    ) -> AgencyClientResult<Vec<u8>> {
        self.inner
            .pack_message(sender_vk, receiver_keys, msg)
            .await
            .map_err(|e| {
                AgencyClientError::from_msg(
                    AgencyClientErrorKind::UnknownError,
                    format!("A VCXError occured while calling pack_message: {:?}", e),
                )
            })
    }

    async fn unpack_message(&self, msg: &[u8]) -> AgencyClientResult<Vec<u8>> {
        self.inner.unpack_message(msg).await.map_err(|e| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::UnknownError,
                format!("A VCXError occured while calling unpack_message: {:?}", e),
            )
        })
    }
}

pub trait ToBaseAgencyClientWallet {
    fn to_base_agency_client_wallet(&self) -> Arc<dyn BaseAgencyClientWallet>;
}

impl ToBaseAgencyClientWallet for Arc<dyn BaseWallet> {
    fn to_base_agency_client_wallet(&self) -> Arc<dyn BaseAgencyClientWallet> {
        Arc::new(BaseWalletAgencyClientWallet {
            inner: Arc::clone(self),
        })
    }
}
