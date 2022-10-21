use async_trait::async_trait;

use crate::error::prelude::AgencyClientResult;

#[async_trait]
pub trait BaseAgencyClientWallet : std::fmt::Debug + Send + Sync {
    async fn pack_message(
        &self,
        sender_vk: Option<&str>,
        receiver_keys: &str,
        msg: &[u8],
    ) -> AgencyClientResult<Vec<u8>>;

    async fn unpack_message(&self, msg: &[u8]) -> AgencyClientResult<Vec<u8>>;
}

#[derive(Debug)]
pub(crate) struct StubAgencyClientWallet;

impl BaseAgencyClientWallet for StubAgencyClientWallet {
    fn pack_message< 'life0, 'life1, 'life2, 'life3, 'async_trait>(& 'life0 self,sender_vk:Option< & 'life1 str> ,receiver_keys: & 'life2 str,msg: & 'life3[u8],) ->  core::pin::Pin<Box<dyn core::future::Future<Output = AgencyClientResult<Vec<u8> > > + core::marker::Send+ 'async_trait> >where 'life0: 'async_trait, 'life1: 'async_trait, 'life2: 'async_trait, 'life3: 'async_trait,Self: 'async_trait {
        todo!()
    }

    fn unpack_message< 'life0, 'life1, 'async_trait>(& 'life0 self,msg: & 'life1[u8]) ->  core::pin::Pin<Box<dyn core::future::Future<Output = AgencyClientResult<Vec<u8> > > + core::marker::Send+ 'async_trait> >where 'life0: 'async_trait, 'life1: 'async_trait,Self: 'async_trait {
        todo!()
    }
}