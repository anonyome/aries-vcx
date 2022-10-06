#[cfg(test)]
#[cfg(feature = "temp_gm_tests")]
mod integration_tests {
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::libindy::utils::pool::PoolConfig;
    use aries_vcx::libindy::utils::signus;
    use aries_vcx::messages::connection::did::Did;
    use aries_vcx::messages::issuance::credential::Credential;
    use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
    use aries_vcx::messages::proof_presentation::presentation_ack::PresentationAck;
    use aries_vcx::messages::proof_presentation::presentation_request::PresentationRequest;
    use aries_vcx::plugins::ledger::base_ledger::BaseLedger;
    use aries_vcx::plugins::ledger::indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerPool};
    use aries_vcx::plugins::wallet::agency_client_wallet::ToBaseAgencyClientWallet;
    use aries_vcx::protocols::issuance::actions::CredentialIssuanceAction;
    use indy_vdr::config::PoolConfig as IndyVdrPoolConfig;
    use indy_vdr::pool::{PoolBuilder, PoolTransactions};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{sync::Arc, thread, time::Duration};

    use agency_client::agency_client::AgencyClient;
    use aries_vcx::messages::a2a::A2AMessage;
    use aries_vcx::protocols::proof_presentation::prover::messages::ProverMessages;
    use aries_vcx::{
        core::profile::{indy_profile::IndySdkProfile, profile::Profile},
        global::{self, settings},
        handlers::connection::connection::Connection,
        libindy::utils::wallet::{create_and_open_wallet, WalletConfig},
        messages::connection::invite::Invitation,
        plugins::wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
        utils::devsetup::{AGENCY_DID, AGENCY_VERKEY},
    };
    use indy_sys::WalletHandle;

    #[tokio::test]
    async fn test_temp() {
        let config_wallet = WalletConfig {
            wallet_name: format!("test_wallet_{}", uuid::Uuid::new_v4().to_string()),
            wallet_key: "helloworld".into(), // settings::DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: settings::WALLET_KDF_DEFAULT.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        let indy_handle = create_and_open_wallet(&config_wallet).await.unwrap();

        let indy_wallet = IndySdkWallet::new(indy_handle);

        let (did, verkey) = indy_wallet.create_and_store_my_did(None, None).await.unwrap();

        println!("{} {}", did, verkey);

        let verkey2 = indy_wallet.get_verkey_from_wallet(&did).await.unwrap();

        println!("{}", verkey2);
    }

    async fn open_default_indy_handle() -> WalletHandle {
        let config_wallet = WalletConfig {
            wallet_name: format!("test_wallet"),
            wallet_key: "helloworld".into(),
            wallet_key_derivation: settings::WALLET_KDF_DEFAULT.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        aries_vcx::libindy::wallet::open_wallet(&config_wallet).await.unwrap()
    }

    fn open_default_agency_client(profile: &Arc<dyn Profile>) -> AgencyClient {
        let mut agency_client = AgencyClient::new();

        agency_client.set_wallet(profile.inject_wallet().to_base_agency_client_wallet());
        agency_client.agency_url = "https://ariesvcx.agency.staging.absa.id".to_string();
        agency_client.agency_did = AGENCY_DID.to_string();
        agency_client.agency_vk = AGENCY_VERKEY.to_string();
        agency_client.agent_pwdid = "XVciZMJAYfn4i5fFNHZ1SC".to_string();
        agency_client.agent_vk = "HcxS4fnkcUy9jfZ5R5v88Rngw3npSLUv17pthXNNCvnz".to_string();
        agency_client.my_pwdid = "12VZYR1AarNNQYAa8iH7WM".to_string();
        agency_client.my_vk = "1pBNDeG2oPEK44zRMEvKn8GbQQpduGVu3QHExBHEPvR".to_string();

        agency_client
    }

    #[tokio::test]
    async fn test_connection() {
        let indy_handle = open_default_indy_handle().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile.clone());

        let agency_client = open_default_agency_client(&profile);

        let invitation = helper::url_to_invitation("https://trinsic.studio/link/?d_m=eyJsYWJlbCI6IkhlbGxvV29ybGQiLCJpbWFnZVVybCI6Imh0dHBzOi8vdHJpbnNpY2FwaWFzc2V0cy5henVyZWVkZ2UubmV0L2ZpbGVzLzc3NTM3ZGU0LWU0YTYtNDUwMS05OTJkLTQ2NGE4MmRiOTFkNl9iYjQwZDUzNC1jNDgyLTQ0YWYtOGQwYS00NmQ5ODQwNzFkZGEucG5nIiwic2VydmljZUVuZHBvaW50IjoiaHR0cHM6Ly9hcGkucG9ydGFsLnN0cmVldGNyZWQuaWQvYWdlbnQva1hmVkhkd2s4MUZKeE40b2lQUHpnaTc2blhUTUY3YzkiLCJyb3V0aW5nS2V5cyI6WyI2cGVLYVV4ZG9yTlVtRVl5Q2JYbXRKWXVhcG1vcDVQUUoyMVh6ZGcxWk1YdCJdLCJyZWNpcGllbnRLZXlzIjpbImpIaEJpQ1huaU1OZTJTQVlZWkE0NjZNNWVvTTUzc1I0RnhRMWROaEcycloiXSwiQGlkIjoiNmI2Yzk1OWItYmRiOC00ZmFjLWE4YjEtMDgzNjkyZGQ4NzE1IiwiQHR5cGUiOiJkaWQ6c292OkJ6Q2JzTlloTXJqSGlxWkRUVUFTSGc7c3BlYy9jb25uZWN0aW9ucy8xLjAvaW52aXRhdGlvbiJ9&orig=https://trinsic.studio/url/3c7ba9ea-265b-4e27-8490-5b27ba029817");
        // invitation.service_endpoint = "http://localhost:8200".to_string();
        let invitation = Invitation::Pairwise(invitation);

        // connect with some default vcx mediator
        // let config_provision_agent = AgentProvisionConfig {
        //     agency_did: AGENCY_DID.to_string(),
        //     agency_verkey: AGENCY_VERKEY.to_string(),
        //     agency_endpoint: "https://ariesvcx.agency.staging.absa.id".to_string(),
        //     agent_seed: None,
        // };
        // provision_cloud_agent(&mut agency_client, &indy_profile, &config_provision_agent).await;

        // println!("agency client; {:?}", agency_client);

        // receive and accept invite

        let autohop = false; // note that trinsic doesn't understand the ACK, so turn it off when using trinisc
        let mut conn = Connection::create_with_invite("69", &profile, &agency_client, invitation, autohop)
            .await
            .unwrap();
        conn.connect(&profile, &agency_client).await.unwrap();

        println!("{:?}", conn.get_state());

        thread::sleep(Duration::from_millis(5000));

        // find response and accept
        // conn.find_message_and_update_state(&profile, &agency_client)
        //     .await
        //     .unwrap();

        // ---- fetch response message and input into state update
        let msgs = conn.get_messages_noauth(&agency_client).await.unwrap();
        let msgs = msgs.iter().collect::<Vec<(&String, &A2AMessage)>>();
        let (msg_id, response_message) = msgs.first().expect("bruh").clone();

        println!("RESAPONSE MESGG: {:?}", response_message);
        conn.update_state_with_message(&profile, agency_client.clone(), Some(response_message.to_owned()))
            .await
            .unwrap();

        println!("{:?}", conn.get_state());
        // remove msg
        conn.update_message_status(msg_id, &agency_client).await.unwrap();
        // check
        assert_eq!(0, conn.get_messages_noauth(&agency_client).await.unwrap().len());

        // ----- send msg

        conn.send_generic_message(&profile, "hellooooo world, ya ya ya")
            .await
            .unwrap();

        println!("{:?}", conn.to_string().unwrap());

        ()
    }

    async fn setup_with_existing_conn() -> (Connection, WalletHandle, IndySdkProfile, Arc<dyn Profile>, AgencyClient) {
        // aca (VCX3)
        let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"X3GT27sVUJi1ExA3FeMLaH\",\"pw_vk\":\"HNbUW7fGRCi3ndrrpJoiLjmjTobuq61ZbEFU9JZw2jt6\",\"agent_did\":\"B43YpHkw3eivvrtCP9MYQ3\",\"agent_vk\":\"6UnAmMw2eMyo31YyxChkayqSNGPTwXAC2ThA76aRxA9z\"},\"state\":{\"Invitee\":{\"Completed\":{\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"did:sov:VNN8vJ3ESb366gX9X1gAGF\",\"publicKey\":[{\"id\":\"did:sov:VNN8vJ3ESb366gX9X1gAGF#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"did:sov:VNN8vJ3ESb366gX9X1gAGF\",\"publicKeyBase58\":\"GTnDMjRozw9eb29xWXN7FGqdmDKCQK2XwZEtTqB1DTh6\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"did:sov:VNN8vJ3ESb366gX9X1gAGF#1\"}],\"service\":[{\"id\":\"did:sov:VNN8vJ3ESb366gX9X1gAGF;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"GTnDMjRozw9eb29xWXN7FGqdmDKCQK2XwZEtTqB1DTh6\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://cloudagent.gmulhearne.di-team.dev.sudoplatform.com:8200\"}]},\"bootstrap_did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"5b287509-8132-4090-ad8f-a96db3399122\",\"publicKey\":[{\"id\":\"5b287509-8132-4090-ad8f-a96db3399122#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"5b287509-8132-4090-ad8f-a96db3399122\",\"publicKeyBase58\":\"Hp7nhx2xU7kcR9SeL4f8gEDfMxM8fCbVSBJDDia9opWA\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"5b287509-8132-4090-ad8f-a96db3399122#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"Hp7nhx2xU7kcR9SeL4f8gEDfMxM8fCbVSBJDDia9opWA\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://cloudagent.gmulhearne.di-team.dev.sudoplatform.com:8200\"}]},\"protocols\":null}}},\"source_id\":\"3\",\"thread_id\":\"5b287509-8132-4090-ad8f-a96db3399122\"}";
        // trin (69)
        // let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"ThFAd68nv6qFcQVhm2LZEp\",\"pw_vk\":\"FYr43xb7eDfhJ8KYgur4BiaDUbF5a7hB1yeFD5Dp48j6\",\"agent_did\":\"VHJzeRk9iufdVcHC4kQSH3\",\"agent_vk\":\"GR2SYM4VGDSnuexUqDRkQHrzJCGnTMK16nx9nsjW6EQK\"},\"state\":{\"Invitee\":{\"Responded\":{\"response\":{\"@id\":\"9c936654-255b-4bba-8838-476a5034f7b9\",\"~thread\":{\"thid\":\"a137601f-bcf6-48bf-ad2f-fd4afd315978\",\"sender_order\":0,\"received_orders\":{}},\"connection~sig\":{\"@type\":\"did:sov:BzCbsNYhMrjHiqZDTUASHg/signature/1.0/ed25519Sha512_single\",\"signature\":\"B2_QxoAfcMo6Dh6-CUbsnA624sokPzsrN3qYWCLSp1fNPH0_yOLe4_w7lAeHYiVZVV-Gii0lP4UMfjN7YcvyDw==\",\"sig_data\":\"i0w2YwAAAAB7IkRJRCI6IlRTV1FOUHM5SlFaU21MY3VGc3JqVlQiLCJESUREb2MiOnsiQGNvbnRleHQiOiJodHRwczovL3czaWQub3JnL2RpZC92MSIsImlkIjoiVFNXUU5QczlKUVpTbUxjdUZzcmpWVCIsInB1YmxpY0tleSI6W3siaWQiOiJUU1dRTlBzOUpRWlNtTGN1RnNyalZUI2tleXMtMSIsInR5cGUiOiJFZDI1NTE5VmVyaWZpY2F0aW9uS2V5MjAxOCIsImNvbnRyb2xsZXIiOiJUU1dRTlBzOUpRWlNtTGN1RnNyalZUIiwicHVibGljS2V5QmFzZTU4IjoiRlFwQkpyZW5OOTlWTFNSWmYxZ0VMUzN1dWY0WjMxM0VkWVR0alFnNVZmYlgifV0sInNlcnZpY2UiOlt7ImlkIjoiVFNXUU5QczlKUVpTbUxjdUZzcmpWVDtpbmR5IiwidHlwZSI6IkluZHlBZ2VudCIsInJlY2lwaWVudEtleXMiOlsiRlFwQkpyZW5OOTlWTFNSWmYxZ0VMUzN1dWY0WjMxM0VkWVR0alFnNVZmYlgiXSwicm91dGluZ0tleXMiOlsiNnBlS2FVeGRvck5VbUVZeUNiWG10Sll1YXBtb3A1UFFKMjFYemRnMVpNWHQiXSwic2VydmljZUVuZHBvaW50IjoiaHR0cHM6Ly9hcGkucG9ydGFsLnN0cmVldGNyZWQuaWQvYWdlbnQva1hmVkhkd2s4MUZKeE40b2lQUHpnaTc2blhUTUY3YzkifV19fQ==\",\"signer\":\"jHhBiCXniMNe2SAYYZA466M5eoM53sR4FxQ1dNhG2rZ\"}},\"request\":{\"@id\":\"a137601f-bcf6-48bf-ad2f-fd4afd315978\",\"label\":\"69\",\"connection\":{\"DID\":\"ThFAd68nv6qFcQVhm2LZEp\",\"DIDDoc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"ThFAd68nv6qFcQVhm2LZEp\",\"publicKey\":[{\"id\":\"ThFAd68nv6qFcQVhm2LZEp#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"ThFAd68nv6qFcQVhm2LZEp\",\"publicKeyBase58\":\"FYr43xb7eDfhJ8KYgur4BiaDUbF5a7hB1yeFD5Dp48j6\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"ThFAd68nv6qFcQVhm2LZEp#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"FYr43xb7eDfhJ8KYgur4BiaDUbF5a7hB1yeFD5Dp48j6\"],\"routingKeys\":[\"GR2SYM4VGDSnuexUqDRkQHrzJCGnTMK16nx9nsjW6EQK\",\"Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR\"],\"serviceEndpoint\":\"https://ariesvcx.agency.staging.absa.id/agency/msg\"}]}},\"~thread\":{\"thid\":\"a137601f-bcf6-48bf-ad2f-fd4afd315978\",\"pthid\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715\",\"sender_order\":0,\"received_orders\":{}},\"~timing\":{\"out_time\":\"2022-09-30T01:55:17.700Z\"}},\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715\",\"publicKey\":[{\"id\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715\",\"publicKeyBase58\":\"jHhBiCXniMNe2SAYYZA466M5eoM53sR4FxQ1dNhG2rZ\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"jHhBiCXniMNe2SAYYZA466M5eoM53sR4FxQ1dNhG2rZ\"],\"routingKeys\":[\"6peKaUxdorNUmEYyCbXmtJYuapmop5PQJ21Xzdg1ZMXt\"],\"serviceEndpoint\":\"https://api.portal.streetcred.id/agent/kXfVHdwk81FJxN4oiPPzgi76nXTMF7c9\"}]}}}},\"source_id\":\"69\",\"thread_id\":\"a137601f-bcf6-48bf-ad2f-fd4afd315978\"}";

        let conn: Connection = Connection::from_string(conn_ser).unwrap();

        let indy_handle = open_default_indy_handle().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile.clone());

        let agency_client = open_default_agency_client(&profile);

        Arc::clone(&profile)
            .inject_anoncreds()
            .prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS)
            .await
            .ok();
        global::pool::open_main_pool(&PoolConfig {
            genesis_path: settings::DEFAULT_GENESIS_PATH.to_string(),
            pool_name: None,
            pool_config: None,
        })
        .await
        .unwrap();

        let precise_time: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        const DAY_SECS: u64 = 60 * 60 * 24;

        aries_vcx::utils::author_agreement::set_txn_author_agreement(
            None,
            Some("1.0".to_string()),
            Some("2f630f02cb1e88d1169db7b4dd0e45943ac1530630d737be5499c8f01c2695b1".to_string()),
            "for_session".to_string(),
            (precise_time / DAY_SECS) * DAY_SECS,
        )
        .unwrap();

        return (conn, indy_handle, indy_profile, profile, agency_client);
    }

    #[tokio::test]
    async fn clear_messages() {
        let (conn, _, _, _, agency_client) = setup_with_existing_conn().await;
        clear_connection_messages(&conn, &agency_client).await;
    }

    #[tokio::test]
    async fn restore_connection() {
        let (conn, _, _, profile, agency_client) = setup_with_existing_conn().await;

        println!("agency client; {:?}", agency_client);

        println!(
            "conn info; {}",
            conn.get_connection_info(&profile, &agency_client).unwrap()
        );

        // conn.send_generic_message(&profile, "hellloooooooooooooooooooooooooo")
        //     .await
        //     .unwrap();

        let msgs = conn.get_messages_noauth(&agency_client).await.unwrap();
        let message = msgs
            .iter()
            .map(|i| i.1.to_owned())
            .collect::<Vec<A2AMessage>>()
            .first()
            .map(|a| a.to_owned());

        println!("MESSAGE!: {:?}", message);
    }

    async fn clear_connection_messages(conn: &Connection, agency_client: &AgencyClient) {
        let msgs = conn.get_messages_noauth(&agency_client).await.unwrap();
        let msgs = msgs.iter().collect::<Vec<(&String, &A2AMessage)>>();

        let len = msgs.len();

        for (msg_id, _) in msgs {
            conn.update_message_status(msg_id, &agency_client).await.unwrap();
        }

        println!("Cleared {:?} messages", len);
    }

    async fn get_first_connection_msg(
        conn: &Connection,
        profile: &Arc<dyn Profile>,
        agency_client: &AgencyClient,
    ) -> (String, A2AMessage) {
        // let msgs = conn.get_messages(profile, &agency_client).await.unwrap();
        let msgs = conn.get_messages_noauth(&agency_client).await.unwrap();
        let msgs = msgs.iter().collect::<Vec<(&String, &A2AMessage)>>();

        let x = msgs.first().expect("no msgs").to_owned();

        (x.0.to_owned(), x.1.to_owned())
    }

    #[tokio::test]
    async fn cred_flow() {
        let (conn, indy_handle, _, profile, agency_client) = setup_with_existing_conn().await;

        println!(
            "{:?}",
            indyrs::anoncreds::prover_get_credentials(indy_handle, None)
                .await
                .unwrap()
        );

        // clear_connection_messages(&conn, &agency_client).await;

        let (msg_id, message) = get_first_connection_msg(&conn, &profile, &agency_client).await;

        println!("MESSAGE!: {:?}", message);

        let offer: CredentialOffer = match message {
            A2AMessage::CredentialOffer(m) => m.to_owned(),
            _ => panic!("aaaa"),
        };

        let mut holder = Holder::create_from_offer("idk", offer).unwrap();

        holder
            .send_request(
                &profile,
                conn.pairwise_info().pw_did.to_string(),
                conn.send_message_closure(&profile).unwrap(),
            )
            .await
            .unwrap();

        // remove msg
        conn.update_message_status(&msg_id, &agency_client).await.unwrap();

        // --------- accept issuance
        //
        println!("sleeping for 10secs");
        thread::sleep(Duration::from_millis(10_000));

        let (msg_id, message) = get_first_connection_msg(&conn, &profile, &agency_client).await;
        println!("MESSAGE!: {:?}", message);

        let issaunce_msg: Credential = match message {
            A2AMessage::Credential(m) => m.to_owned(),
            _ => panic!("aaaa"),
        };

        holder
            .step(
                &profile,
                CredentialIssuanceAction::Credential(issaunce_msg),
                Some(conn.send_message_closure(&profile).unwrap()),
            )
            .await
            .unwrap();

        println!("state; {:?}", holder.get_state());
        println!("cred; {:?}", holder.get_credential());
        println!("whole; {:?}", holder);

        // remove msg
        conn.update_message_status(&msg_id, &agency_client).await.unwrap();

        ()
    }

    #[tokio::test]
    async fn proof_pres() {
        let (conn, indy_handle, indy_profile, profile, agency_client) = setup_with_existing_conn().await;

        println!(
            "{:?}",
            Arc::clone(&profile)
                .inject_anoncreds()
                .prover_get_credentials(None)
                .await
                .unwrap()
        );

        let (msg_id, message) = get_first_connection_msg(&conn, &profile, &agency_client).await;

        let pres_req: PresentationRequest = match message {
            A2AMessage::PresentationRequest(m) => m.to_owned(),
            _ => panic!("aaaa"),
        };

        let mut prover = Prover::create_from_request("1", pres_req).unwrap();

        conn.update_message_status(&msg_id, &agency_client).await.unwrap();

        let creds = prover.retrieve_credentials(&profile).await.unwrap();
        println!("creds; {:?}", creds);

        let credentials: HashMap<String, serde_json::Value> = serde_json::from_str(&creds).unwrap();

        let mut use_credentials = serde_json::json!({});

        for (referent, credentials) in credentials["attrs"].as_object().unwrap().iter() {
            let cred_rev_reg_id = credentials[0]
                .get("cred_info")
                .and_then(|v| v.get("rev_reg_id"))
                .and_then(|v| v.as_str())
                .unwrap();

            let (_, rev_reg_def_json) = Arc::clone(&profile)
                .inject_anoncreds()
                .get_rev_reg_def_json(cred_rev_reg_id)
                .await
                .unwrap();

            let rev_reg_def: Value = serde_json::from_str(&rev_reg_def_json).unwrap();

            let tails_location = rev_reg_def
                .get("value")
                .and_then(|value| value.get("tailsLocation"))
                .and_then(Value::as_str)
                .unwrap();
            let tails_hash = rev_reg_def
                .get("value")
                .and_then(|value| value.get("tailsHash"))
                .and_then(Value::as_str)
                .unwrap();

            let tails_file = helper::download_tails(tails_hash, tails_location).await;

            use_credentials["attrs"][referent] = serde_json::json!({
                "credential": credentials[0],
                "tails_file": tails_file
            })
        }

        println!("creds to use; {:?}", use_credentials.to_string());

        prover
            .generate_presentation(&profile, use_credentials.to_string(), "{}".to_string())
            .await
            .unwrap();

        prover
            .send_presentation(&profile, conn.send_message_closure(&profile).unwrap())
            .await
            .unwrap();

        println!("sleeping for 20secs for ACK - GO VERIFY IT!");
        thread::sleep(Duration::from_millis(20_000));

        let (msg_id, message) = get_first_connection_msg(&conn, &profile, &agency_client).await;
        println!("MESSAGE!: {:?}", message);

        let pres_ack: PresentationAck = match message {
            A2AMessage::PresentationAck(m) => m.to_owned(),
            _ => panic!("aaaa"),
        };

        prover
            .handle_message(
                &profile,
                ProverMessages::PresentationAckReceived(pres_ack),
                Some(conn.send_message_closure(&profile).unwrap()),
            )
            .await
            .unwrap();

        println!("{:?}", prover);
        println!("{:?}", prover.presentation_status());

        conn.update_message_status(&msg_id, &agency_client).await.unwrap();

        ()
    }

    #[tokio::test]
    async fn test_ledger_fetch() {
        // ----------- try with indyvdr
        let (_ack, indy_handle, _, _, _) = setup_with_existing_conn().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile); // just for the wallet

        // let pool_runner = PoolRunner::new(config, merkle_tree, networker_factory, None);
        let config = IndyVdrPoolConfig::default();
        let txns = PoolTransactions::from_json_file(
            "/Users/gmulhearne/Documents/dev/platform/di-edge-agent/edge-agent-core/aries-vcx/aries_vcx/genesis.txn",
        )
        .unwrap();

        let runner = PoolBuilder::from(config)
            .transactions(txns)
            .unwrap()
            .into_runner()
            .unwrap();
        let indy_vdr_pool = IndyVdrLedgerPool::new(runner);
        let ledger = IndyVdrLedger::new(profile, indy_vdr_pool);

        let x = ledger.get_nym("D6EMVkDnBmuMCtZGwjgR9A").await.unwrap();

        println!("VDR NYM: {}\n\n\n", x);

        let y = ledger
            .get_cred_def("D6EMVkDnBmuMCtZGwjgR9A:3:CL:88813:Dummy_Uni_Transaction")
            .await
            .unwrap();

        println!("VDR CRED DEF: {}\n\n\n", y);

        // ----------- try with indy

        println!(
            "INDY NYM: {}\n\n\n",
            aries_vcx::libindy::utils::ledger::get_nym("D6EMVkDnBmuMCtZGwjgR9A")
                .await
                .unwrap()
        );

        println!(
            "INDY CRED DEF: {}\n\n\n",
            aries_vcx::libindy::utils::ledger::libindy_get_cred_def(
                indy_handle,
                "D6EMVkDnBmuMCtZGwjgR9A:3:CL:88813:Dummy_Uni_Transaction"
            )
            .await
            .unwrap()
        );

        let pub_did = "D6EMVkDnBmuMCtZGwjgR9A";
        let pub_verkey = signus::get_verkey_from_wallet(indy_handle, pub_did).await.unwrap();
        // let x = ledger
        //     .add_service(
        //         &pub_did,
        //         &AriesService {
        //             id: "idk".to_string(),
        //             type_: "idk".to_string(),
        //             priority: 0,
        //             recipient_keys: vec![pub_verkey],
        //             routing_keys: vec![],
        //             service_endpoint: "http://hello.world".to_string(),
        //         },
        //     )
        //     .await
        //     .unwrap();

        println!(
            "service; {:?}",
            ledger
                .get_service(&Did::new("D6EMVkDnBmuMCtZGwjgR9A").unwrap())
                .await
                .unwrap()
        );

        ()
    }

    mod helper {

        use std::{fs::File, io::Write, path::Path};

        use agency_client::{
            agency_client::AgencyClient,
            configuration::{AgencyClientConfig, AgentProvisionConfig},
        };
        use aries_vcx::{
            core::profile::{indy_profile::IndySdkProfile, profile::Profile},
            messages::connection::invite::PairwiseInvitation,
            plugins::wallet::agency_client_wallet::ToBaseAgencyClientWallet,
        };
        use url::Url;

        pub fn url_to_invitation(url: &str) -> PairwiseInvitation {
            let b64_val = Url::parse(url)
                .unwrap()
                .query_pairs()
                .find(|(x, _)| x == "c_i" || x == "d_m")
                .unwrap()
                .1
                .to_string();

            let v = String::from_utf8(base64::decode_config(&b64_val, base64::URL_SAFE).unwrap()).unwrap();

            serde_json::from_str(&v).unwrap()
        }

        pub async fn provision_cloud_agent(
            client: &mut AgencyClient,
            profile: &IndySdkProfile,
            provision_config: &AgentProvisionConfig,
        ) -> AgencyClientConfig {
            let seed = provision_config.agent_seed.as_deref();
            let (my_did, my_vk) = profile
                .inject_wallet()
                .create_and_store_my_did(seed, None)
                .await
                .unwrap();

            client
                .provision_cloud_agent(
                    profile.inject_wallet().to_base_agency_client_wallet(),
                    &my_did,
                    &my_vk,
                    &provision_config.agency_did,
                    &provision_config.agency_verkey,
                    &provision_config.agency_endpoint,
                )
                .await
                .unwrap();
            let config = client.get_config().unwrap();

            config
        }

        pub async fn download_tails(hash: &str, tails_location: &str) -> String {
            let file_path = format!(
                "/Users/gmulhearne/Documents/dev/platform/di-edge-agent/edge-agent-core/aries-vcx/aries_vcx/tails/{}",
                hash
            );

            let path = Path::new(&file_path);

            let parent_dir = path.parent().unwrap().to_str().unwrap().to_string();

            if path.exists() {
                return parent_dir;
            }

            let mut tails_file = File::create(path).unwrap();

            let x = reqwest::get(tails_location).await.unwrap();

            let bs = x.bytes().await.unwrap();

            tails_file.write(&bs).unwrap();

            tails_file.flush().unwrap();

            file_path
        }
    }
}
