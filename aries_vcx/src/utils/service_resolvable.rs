use std::sync::Arc;

use crate::core::profile::profile::Profile;
use crate::did_doc::service_aries::AriesService;
use crate::error::prelude::*;
use crate::messages::connection::did::Did;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServiceResolvable {
    AriesService(AriesService),
    Did(Did),
}

impl ServiceResolvable {
    pub async fn resolve(&self, profile: &Arc<dyn Profile>) -> VcxResult<AriesService> {
        let ledger = Arc::clone(profile).inject_ledger();
        match self {
            ServiceResolvable::AriesService(service) => Ok(service.clone()),
            ServiceResolvable::Did(did) => ledger.get_service(did).await,
        }
    }
}
