use std::sync::Arc;

use messages::{did_doc::{service_resolvable::ServiceResolvable, service_aries::AriesService, DidDoc}, connection::invite::Invitation};

use crate::{core::profile::profile::Profile, error::VcxResult};

pub async fn resolve_service(profile: &Arc<dyn Profile>, service: &ServiceResolvable) -> VcxResult<AriesService> {
    let ledger = Arc::clone(profile).inject_ledger();
    match service {
        ServiceResolvable::AriesService(service) => Ok(service.clone()),
        ServiceResolvable::Did(did) => ledger.get_service(did).await,
    }
}


pub async fn into_did_doc(profile: &Arc<dyn Profile>, invitation: &Invitation) -> VcxResult<DidDoc> {
    let ledger = Arc::clone(profile).inject_ledger();
    let mut did_doc: DidDoc = DidDoc::default();
    let (service_endpoint, recipient_keys, routing_keys) = match invitation {
        Invitation::Public(invitation) => {
            did_doc.set_id(invitation.did.to_string());
            let service = ledger.get_service(&invitation.did).await.unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {}", err);
                AriesService::default()
            });
            (service.service_endpoint, service.recipient_keys, service.routing_keys)
        }
        Invitation::Pairwise(invitation) => {
            did_doc.set_id(invitation.id.0.clone());
            (
                invitation.service_endpoint.clone(),
                invitation.recipient_keys.clone(),
                invitation.routing_keys.clone(),
            )
        }
        Invitation::OutOfBand(invitation) => {
            did_doc.set_id(invitation.id.0.clone());
            let service = resolve_service(profile, &invitation.services[0]).await.unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {}", err);
                AriesService::default()
            });
            (service.service_endpoint, service.recipient_keys, service.routing_keys)
        }
    };
    did_doc.set_service_endpoint(service_endpoint);
    did_doc.set_recipient_keys(recipient_keys);
    did_doc.set_routing_keys(routing_keys);
    Ok(did_doc)
}