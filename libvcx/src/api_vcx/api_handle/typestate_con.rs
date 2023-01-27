use std::{collections::HashMap, sync::RwLock};

use agency_client::httpclient::post_message;
use aries_vcx::{
    errors::error::VcxResult,
    protocols::typestate_con::{
        invitee::InviteeConnection, inviter::InviterConnection, pairwise_info::PairwiseInfo, Connection, Transport,
        VagueConnection,
    },
};
use async_trait::async_trait;
use rand::Rng;

use crate::{
    api_vcx::api_global::profile::get_main_profile,
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

type Map = HashMap<u32, VagueConnection>;
type Cache = RwLock<Map>;

lazy_static! {
    static ref CONNECTION_MAP: Cache = RwLock::new(HashMap::new());
}

lazy_static! {
    static ref HTTP_CLIENT: HttpClient = HttpClient;
}

struct HttpClient;

#[async_trait]
impl Transport for HttpClient {
    async fn send_message(&self, msg: Vec<u8>, service_endpoint: &str) -> VcxResult<()> {
        post_message(msg, service_endpoint).await?;
        Ok(())
    }
}

fn new_handle() -> LibvcxResult<u32> {
    loop {
        let handle = rand::thread_rng().gen::<u32>();
        if !CONNECTION_MAP.read()?.contains_key(&handle) {
            break Ok(handle);
        }
    }
}

fn remove_connection<I, S>(handle: &u32) -> LibvcxResult<Connection<I, S>>
where
    Connection<I, S>: TryFrom<VagueConnection>,
{
    CONNECTION_MAP
        .write()?
        .remove(handle)
        .and_then(|c| c.try_into().ok())
        .ok_or_else(|| {
            LibvcxError::from_msg(
                LibvcxErrorKind::ObjectAccessError,
                format!("Unable to retrieve expected connection for handle: {}", handle),
            )
        })
}

fn add_connection(connection: VagueConnection) -> LibvcxResult<u32> {
    let handle = new_handle()?;
    CONNECTION_MAP.write()?.insert(handle, connection);
    Ok(handle)
}

fn insert_connection<I, S>(handle: u32, connection: Connection<I, S>) -> LibvcxResult<()>
where
    VagueConnection: From<Connection<I, S>>,
{
    CONNECTION_MAP.write()?.insert(handle, connection.into());
    Ok(())
}

fn serialize<T>(data: &T) -> LibvcxResult<String>
where
    T: serde::ser::Serialize,
{
    serde_json::to_string(data).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::SerializationError,
            format!("Serialization failed: {}", err),
        )
    })
}

fn deserialize<T>(data: &str) -> LibvcxResult<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(data)
        .map_err(|err| LibvcxError::from_msg(LibvcxErrorKind::InvalidJson, format!("Deserialization failed: {}", err)))
}

// ----------------------------- CONSTRUCTORS ------------------------------------
pub async fn create_inviter(pw_info: Option<PairwiseInfo>) -> LibvcxResult<u32> {
    trace!("create_inviter >>>");
    let profile = get_main_profile()?;

    // This could probably be generated once and used/cloned around.
    let pw_info = pw_info.unwrap_or(PairwiseInfo::create(&profile.inject_wallet()).await?);
    let con = InviterConnection::new_invited("".to_owned(), pw_info);

    add_connection(con.into())
}

pub async fn create_invitee(invitation: &str) -> LibvcxResult<u32> {
    trace!("create_invitee >>>");

    let profile = get_main_profile()?;
    let invitation = deserialize(invitation)?;
    let pairwise_info = PairwiseInfo::create(&profile.inject_wallet()).await?;

    let con = InviteeConnection::new("".to_owned(), pairwise_info)
        .accept_invitation(&profile, &invitation)
        .await?;

    add_connection(con.into())
}

// Just trying to retro-fit this.
// It essentially creates an inviter connection in the initial state, also genereting an Invitation.
pub async fn create_invite(handle: u32, service_endpoint: String, routing_keys: Vec<String>) -> LibvcxResult<()> {
    trace!("create_invite >>>");

    let profile = get_main_profile()?;
    let pairwise_info = PairwiseInfo::create(&profile.inject_wallet()).await?;
    let con = InviterConnection::new("".to_owned(), pairwise_info, routing_keys, service_endpoint);

    insert_connection(handle, con)
}

// ----------------------------- GETTERS ------------------------------------
// pub fn get_thread_id(handle: u32) -> LibvcxResult<String> {
//     trace!("get_thread_id >>> handle: {}", handle);
//     CONNECTION_MAP.get(handle, |connection| Ok(connection.get_thread_id()))
// }

// pub fn get_pairwise_info(handle: u32) -> LibvcxResult<String> {
//     trace!("get_pairwise_info >>> handle: {}", handle);
//     CONNECTION_MAP.get(handle, |connection| serialize(connection.pairwise_info()))
// }

// pub fn get_remote_did(handle: u32) -> LibvcxResult<String> {
//     trace!("get_remote_did >>> handle: {}", handle);
//     CONNECTION_MAP.get(handle, |connection| connection.remote_did().map_err(|e| e.into()))
// }

// pub fn get_remote_vk(handle: u32) -> LibvcxResult<String> {
//     trace!("get_remote_vk >>> handle: {}", handle);
//     CONNECTION_MAP.get(handle, |connection| connection.remote_vk().map_err(|e| e.into()))
// }

// pub fn get_state(handle: u32) -> LibvcxResult<u32> {
//     trace!("get_state >>> handle: {}", handle);
//     CONNECTION_MAP.get(handle, |connection| Ok(connection.get_state().into()))
// }

// pub fn get_invitation(handle: u32) -> LibvcxResult<String> {
//     trace!("get_invitation >>> handle: {}", handle);

//     let invitation = get_connection(&handle)?.get_invitation();
//     serialize(invitation)
// }

// ----------------------------- MSG PROCESSING ------------------------------------
pub async fn process_invite(handle: u32, invitation: &str) -> LibvcxResult<()> {
    trace!("process_invite >>>");

    // ------------------ BREAKING CHANGE ------------------
    // The code below would do the proper thing, but the function must be async
    // Also, this doesn't really make sense conceptually as the invitation is also passed
    // when the invitee is created -> so the transition happens there automatically.

    let profile = get_main_profile()?;
    let invitation = deserialize(invitation)?;
    let con = remove_connection(&handle)?
        .accept_invitation(&profile, &invitation)
        .await?;

    insert_connection(handle, con)
}

// ------------------ BREAKING CHANGE ------------------
// Extra function arguments are deffered to send_response.
pub async fn process_request(handle: u32, request: &str) -> LibvcxResult<()> {
    trace!("process_request >>>");

    let con = remove_connection(&handle)?;
    let request = deserialize(request)?;
    let con = con.handle_request(request).await?;

    insert_connection(handle, con)
}

pub async fn process_response(handle: u32, response: &str) -> LibvcxResult<()> {
    trace!("process_response >>>");

    let con = remove_connection(&handle)?;
    let wallet = get_main_profile()?.inject_wallet();
    let response = deserialize(response)?;
    let con = con.handle_response(&wallet, response).await?;

    insert_connection(handle, con)
}

pub async fn process_ack(handle: u32, message: &str) -> LibvcxResult<()> {
    trace!("process_ack >>>");

    let con = remove_connection(&handle)?;
    let msg = deserialize(message)?;
    let con = con.acknowledge_connection(&msg)?;

    insert_connection(handle, con)
}

// ------------------ BREAKING CHANGE ------------------
// Got new arguments that were moved from process_request
pub async fn send_response(handle: u32, service_endpoint: String, routing_keys: Vec<String>) -> LibvcxResult<()> {
    trace!("send_response >>>");

    let con = remove_connection(&handle)?;
    let wallet = get_main_profile()?.inject_wallet();
    let con = con
        .send_response(&wallet, service_endpoint, routing_keys, &*HTTP_CLIENT)
        .await?;

    insert_connection(handle, con)
}

pub async fn send_request(handle: u32, service_endpoint: String, routing_keys: Vec<String>) -> LibvcxResult<()> {
    trace!("send_request >>>");

    let con = remove_connection(&handle)?;
    let wallet = get_main_profile()?.inject_wallet();
    let con = con
        .send_request(&wallet, service_endpoint, routing_keys, &*HTTP_CLIENT)
        .await?;

    insert_connection(handle, con)
}

pub async fn send_ack(handle: u32) -> LibvcxResult<()> {
    trace!("send_ack >>>");

    let con = remove_connection(&handle)?;
    let wallet = get_main_profile()?.inject_wallet();
    let con = con.send_ack(&wallet, &*HTTP_CLIENT).await?;

    insert_connection(handle, con)
}

// // ------------------------- (DE)SERIALIZATION ----------------------------------
pub fn to_string(handle: u32) -> LibvcxResult<String> {
    trace!("to_string >>>");

    CONNECTION_MAP
        .read()?
        .get(&handle)
        .ok_or_else(|| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidHandle,
                format!("[Connection Cache] get >> Object not found for handle: {}", handle),
            )
        })
        .and_then(serialize)
}

pub fn from_string(connection_data: &str) -> LibvcxResult<u32> {
    trace!("from_string >>>");
    add_connection(deserialize(connection_data)?)
}

// ------------------------------ CLEANUP ---------------------------------------
pub fn release(handle: u32) -> LibvcxResult<()> {
    trace!("release >>>");

    CONNECTION_MAP.write().map(|mut map| map.remove(&handle)).ok();
    Ok(())
}

pub fn release_all() {
    trace!("release_all >>>");
    CONNECTION_MAP.write().map(|mut map| map.drain().for_each(drop)).ok();
}
