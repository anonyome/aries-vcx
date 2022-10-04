use std::sync::Arc;

use crate::core::profile::profile::Profile;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::request::Request;
use crate::protocols::connection::invitee::states::requested::RequestedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: Invitation,
}

impl From<(InvitedState, Request, &Arc<dyn Profile>)> for RequestedState {
    fn from((state, request, profile): (InvitedState, Request, &Arc<dyn Profile>)) -> RequestedState {
        trace!("ConnectionInvitee: transit state from InvitedState to RequestedState");
        RequestedState {
            request,
            did_doc: state.invitation.resolve_did_doc(profile),
        }
    }
}
