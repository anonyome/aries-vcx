#[cfg(test)]
#[cfg(feature = "wallet_tests")]
mod integration_tests {
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::libindy::utils::pool::PoolConfig;
    use aries_vcx::messages::issuance::credential::Credential;
    use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
    use aries_vcx::messages::proof_presentation::presentation_ack::PresentationAck;
    use aries_vcx::messages::proof_presentation::presentation_request::PresentationRequest;
    use aries_vcx::protocols::issuance::actions::CredentialIssuanceAction;
    use aries_vcx::wallet::agency_client_wallet::ToBaseAgencyClientWallet;
    use std::collections::HashMap;
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
        utils::devsetup::{AGENCY_DID, AGENCY_VERKEY},
        wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
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

        let invitation = helper::url_to_invitation("http://cloudagent.gmulhearne.di-team.dev.sudoplatform.com:8200?c_i=eyJAdHlwZSI6ICJkaWQ6c292OkJ6Q2JzTlloTXJqSGlxWkRUVUFTSGc7c3BlYy9jb25uZWN0aW9ucy8xLjAvaW52aXRhdGlvbiIsICJAaWQiOiAiZmI5ZWM5YmMtMmM1Mi00ZDgxLWJhNmEtY2MxOTU4MzFhNWIxIiwgInJlY2lwaWVudEtleXMiOiBbIkE1QVl3Nk5WVUJpM3NEVEQ5Q3ZmUkhKQ29iOFB0Qkg3YVllRjlVWTk1aHZDIl0sICJzZXJ2aWNlRW5kcG9pbnQiOiAiaHR0cDovL2Nsb3VkYWdlbnQuZ211bGhlYXJuZS5kaS10ZWFtLmRldi5zdWRvcGxhdGZvcm0uY29tOjgyMDAiLCAibGFiZWwiOiAiZ211bGhlYXJuZSJ9");
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

        println!("agency client; {:?}", agency_client);

        // receive and accept invite

        let autohop = true; // note that trinsic doesn't understand the ACK, so turn it off when using trinisc
        let mut conn = Connection::create_with_invite("3", &profile, &agency_client, invitation, autohop)
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
        let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"3eACR14SizUPywULrydBzq\",\"pw_vk\":\"2SUeN3GHWZVnoRtBrxm691aEj3girWtZQqfgnwse1NKf\",\"agent_did\":\"DQEjA6wm3WkJLpZ8SFP7np\",\"agent_vk\":\"7m1VN5LA5LSA6mJzHViPPHc8S6DwesapTv8LYUgo1NM7\"},\"state\":{\"Invitee\":{\"Completed\":{\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"did:sov:Ni9cSfBEwpCxZgAYAit4iE\",\"publicKey\":[{\"id\":\"did:sov:Ni9cSfBEwpCxZgAYAit4iE#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"did:sov:Ni9cSfBEwpCxZgAYAit4iE\",\"publicKeyBase58\":\"CqHAb5ucwo9bsJN66J1P4KjvTdXqt5JEw6fMhwvz626k\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"did:sov:Ni9cSfBEwpCxZgAYAit4iE#1\"}],\"service\":[{\"id\":\"did:sov:Ni9cSfBEwpCxZgAYAit4iE;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"CqHAb5ucwo9bsJN66J1P4KjvTdXqt5JEw6fMhwvz626k\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://cloudagent.gmulhearne.di-team.dev.sudoplatform.com:8200\"}]},\"bootstrap_did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"fb9ec9bc-2c52-4d81-ba6a-cc195831a5b1\",\"publicKey\":[{\"id\":\"fb9ec9bc-2c52-4d81-ba6a-cc195831a5b1#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"fb9ec9bc-2c52-4d81-ba6a-cc195831a5b1\",\"publicKeyBase58\":\"A5AYw6NVUBi3sDTD9CvfRHJCob8PtBH7aYeF9UY95hvC\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"fb9ec9bc-2c52-4d81-ba6a-cc195831a5b1#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"A5AYw6NVUBi3sDTD9CvfRHJCob8PtBH7aYeF9UY95hvC\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://cloudagent.gmulhearne.di-team.dev.sudoplatform.com:8200\"}]},\"protocols\":null}}},\"source_id\":\"3\",\"thread_id\":\"acf3ec1a-39e0-481a-92c7-b126c3aa123e\"}";

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

        return (conn, indy_handle, indy_profile, profile, agency_client);
    }


    #[tokio::test]
    async fn clear_messages() {
        let (conn, _, _, _, agency_client) = setup_with_existing_conn().await;
        clear_connection_messages(&conn, &agency_client).await;
    }

    #[tokio::test]
    async fn restore_connection() {
        let (conn, _, _, _, agency_client) = setup_with_existing_conn().await;

        println!("agency client; {:?}", agency_client);

        println!("conn info; {}", conn.get_connection_info(&agency_client).unwrap());

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

        println!("Cleared {:?} messages",len);
    }

    async fn get_first_connection_msg(conn: &Connection, agency_client: &AgencyClient) -> (String, A2AMessage) {
        let msgs = conn.get_messages(&agency_client).await.unwrap();
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

        let (msg_id, message) = get_first_connection_msg(&conn, &agency_client).await;

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

        let (msg_id, message) = get_first_connection_msg(&conn, &agency_client).await;
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

        let (msg_id, message) = get_first_connection_msg(&conn, &agency_client).await;

        let pres_req: PresentationRequest = match message {
            A2AMessage::PresentationRequest(m) => m.to_owned(),
            _ => panic!("aaaa"),
        };

        let mut prover = Prover::create_from_request("1", pres_req).unwrap();

        println!("prover; {:?}", prover);

        conn.update_message_status(&msg_id, &agency_client).await.unwrap();

        let creds = prover.retrieve_credentials(&profile).await.unwrap();
        println!("creds; {:?}", creds);

        let credentials: HashMap<String, serde_json::Value> = serde_json::from_str(&creds).unwrap();

        let mut use_credentials = serde_json::json!({});

        for (referent, credentials) in credentials["attrs"].as_object().unwrap().iter() {
            use_credentials["attrs"][referent] = serde_json::json!({
                "credential": credentials[0]
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

        println!("{:?}", prover.presentation_status());

        println!("sleeping for 20secs for ACK - GO VERIFY IT!");
        thread::sleep(Duration::from_millis(20_000));

        let (msg_id, message) = get_first_connection_msg(&conn, &agency_client).await;
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

        ()
    }

    mod helper {

        use agency_client::{
            agency_client::AgencyClient,
            configuration::{AgencyClientConfig, AgentProvisionConfig},
        };
        use aries_vcx::{
            core::profile::{indy_profile::IndySdkProfile, profile::Profile},
            messages::connection::invite::PairwiseInvitation,
            wallet::agency_client_wallet::ToBaseAgencyClientWallet,
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
    }
}
