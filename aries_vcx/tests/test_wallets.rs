#[cfg(test)]
#[cfg(feature = "wallet_tests")]
mod integration_tests {
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::libindy::utils::pool::PoolConfig;
    use aries_vcx::messages::issuance::credential::Credential;
    use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
    use aries_vcx::protocols::issuance::actions::CredentialIssuanceAction;
    use aries_vcx::wallet::agency_client_wallet::ToBaseAgencyClientWallet;
    use std::{sync::Arc, thread, time::Duration};

    use agency_client::{agency_client::AgencyClient, configuration::AgentProvisionConfig};
    use aries_vcx::messages::a2a::A2AMessage;
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

        let invitation = helper::url_to_invitation("http://1aca-125-253-16-164.ngrok.io?c_i=eyJAdHlwZSI6ICJkaWQ6c292OkJ6Q2JzTlloTXJqSGlxWkRUVUFTSGc7c3BlYy9jb25uZWN0aW9ucy8xLjAvaW52aXRhdGlvbiIsICJAaWQiOiAiYzBiYjdiMmItZWFhYy00M2JjLWFkZDMtYmE2Y2NjN2I5MTYwIiwgImxhYmVsIjogIkFyaWVzIENsb3VkIEFnZW50IiwgInNlcnZpY2VFbmRwb2ludCI6ICJodHRwOi8vMWFjYS0xMjUtMjUzLTE2LTE2NC5uZ3Jvay5pbyIsICJyZWNpcGllbnRLZXlzIjogWyJBQ2FKS1N1WHlQTm8yU2t3ejU3RE5mRGh1NUZwdXlSM2dQb0F4VVFVUTNHbiJdfQ==");
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
        
        let autohop = false; // note that trinsic doesn't understand the ACK, so turn it off
        let mut conn = Connection::create_with_invite("1", &profile, &agency_client, invitation, autohop)
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

    #[tokio::test]
    async fn restore_connection() {
        let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"7ePNQN9QV8NTym41ct724a\",\"pw_vk\":\"4d3oT5pBkXx6SVMQYAMPZp6hmVTAbozRUcq7KZWMAmYY\",\"agent_did\":\"Qu1CWMtxGrmizu48bphT8E\",\"agent_vk\":\"E2RJRmmYwipWgXpHbYap1Dc9Dh6qAxDYNuRMjwAEJcJU\"},\"state\":{\"Invitee\":{\"Responded\":{\"response\":{\"@id\":\"a3309155-4de6-4c4d-8206-4b4b099b4b2c\",\"~thread\":{\"thid\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\",\"sender_order\":0,\"received_orders\":{}},\"connection~sig\":{\"@type\":\"did:sov:BzCbsNYhMrjHiqZDTUASHg/signature/1.0/ed25519Sha512_single\",\"signature\":\"TY-024HPg8eXOMrettwUXF3MhRgOkeZB3htBKx4Yyp9PB0JS0_05BJGj93InUg0PrFHOeiLn1z8K9Ojhp4RGDw==\",\"sig_data\":\"AAAAAGMxSf97IkRJRCI6ICIybVRHSmRoMTdKSENBSE10dDRkRndhIiwgIkRJRERvYyI6IHsiQGNvbnRleHQiOiAiaHR0cHM6Ly93M2lkLm9yZy9kaWQvdjEiLCAiaWQiOiAiZGlkOnNvdjoybVRHSmRoMTdKSENBSE10dDRkRndhIiwgInB1YmxpY0tleSI6IFt7ImlkIjogImRpZDpzb3Y6Mm1UR0pkaDE3SkhDQUhNdHQ0ZEZ3YSMxIiwgInR5cGUiOiAiRWQyNTUxOVZlcmlmaWNhdGlvbktleTIwMTgiLCAiY29udHJvbGxlciI6ICJkaWQ6c292OjJtVEdKZGgxN0pIQ0FITXR0NGRGd2EiLCAicHVibGljS2V5QmFzZTU4IjogInhxamdBS2U3VXpNMkpUd0RNRjV5OWRlWGNLcXo4SGNaTG02OENmaFJVcGsifV0sICJhdXRoZW50aWNhdGlvbiI6IFt7InR5cGUiOiAiRWQyNTUxOVNpZ25hdHVyZUF1dGhlbnRpY2F0aW9uMjAxOCIsICJwdWJsaWNLZXkiOiAiZGlkOnNvdjoybVRHSmRoMTdKSENBSE10dDRkRndhIzEifV0sICJzZXJ2aWNlIjogW3siaWQiOiAiZGlkOnNvdjoybVRHSmRoMTdKSENBSE10dDRkRndhO2luZHkiLCAidHlwZSI6ICJJbmR5QWdlbnQiLCAicHJpb3JpdHkiOiAwLCAicmVjaXBpZW50S2V5cyI6IFsieHFqZ0FLZTdVek0ySlR3RE1GNXk5ZGVYY0txejhIY1pMbTY4Q2ZoUlVwayJdLCAic2VydmljZUVuZHBvaW50IjogImh0dHA6Ly8xYWNhLTEyNS0yNTMtMTYtMTY0Lm5ncm9rLmlvIn1dfX0=\",\"signer\":\"ACaJKSuXyPNo2Skwz57DNfDhu5FpuyR3gPoAxUQUQ3Gn\"}},\"request\":{\"@id\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\",\"label\":\"1\",\"connection\":{\"DID\":\"7ePNQN9QV8NTym41ct724a\",\"DIDDoc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"7ePNQN9QV8NTym41ct724a\",\"publicKey\":[{\"id\":\"7ePNQN9QV8NTym41ct724a#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"7ePNQN9QV8NTym41ct724a\",\"publicKeyBase58\":\"4d3oT5pBkXx6SVMQYAMPZp6hmVTAbozRUcq7KZWMAmYY\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"7ePNQN9QV8NTym41ct724a#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"4d3oT5pBkXx6SVMQYAMPZp6hmVTAbozRUcq7KZWMAmYY\"],\"routingKeys\":[\"E2RJRmmYwipWgXpHbYap1Dc9Dh6qAxDYNuRMjwAEJcJU\",\"Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR\"],\"serviceEndpoint\":\"https://ariesvcx.agency.staging.absa.id/agency/msg\"}]}},\"~thread\":{\"thid\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\",\"sender_order\":0,\"received_orders\":{}},\"~timing\":{\"out_time\":\"2022-09-26T06:43:11.142Z\"}},\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160\",\"publicKey\":[{\"id\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160\",\"publicKeyBase58\":\"ACaJKSuXyPNo2Skwz57DNfDhu5FpuyR3gPoAxUQUQ3Gn\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"ACaJKSuXyPNo2Skwz57DNfDhu5FpuyR3gPoAxUQUQ3Gn\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://1aca-125-253-16-164.ngrok.io\"}]}}}},\"source_id\":\"1\",\"thread_id\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\"}";

        let conn: Connection = Connection::from_string(conn_ser).unwrap();

        let indy_handle = open_default_indy_handle().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile.clone());

        let agency_client = open_default_agency_client(&profile);

        //agency client; AgencyClient { wallet: BaseWalletAgencyClientWallet { inner: IndySdkWallet { handle: WalletHandle(2) } }, agency_url: "https://ariesvcx.agency.staging.absa.id", agency_did: "VsKV7grR1BUE29mG2Fm2kX", agency_vk: "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR", agent_pwdid: "R9QvejvJz2n4DQ8wo4PNT2", agent_vk: "EAGoPpLWNNwC3de2htXvKjN8VX2ZgVuC9WASfq5oK4nD", my_pwdid: "Age2B8HDZoCNiVQ1yLiGYL", my_vk: "6H7VcGf9cz5V2nDo3MisXby3usKxdsgejsHZg2NFX1VD" }

        // agency_client.set_wallet(indy_profile.inject_wallet().to_base_agency_client_wallet());
        // agency_client.agency_url = "https://ariesvcx.agency.staging.absa.id".to_string();
        // agency_client.agency_did = AGENCY_DID.to_string();
        // agency_client.agency_vk = AGENCY_VERKEY.to_string();
        // agency_client.agent_pwdid = "XVciZMJAYfn4i5fFNHZ1SC".to_string();
        // agency_client.agent_vk = "HcxS4fnkcUy9jfZ5R5v88Rngw3npSLUv17pthXNNCvnz".to_string();
        // agency_client.my_pwdid = "12VZYR1AarNNQYAa8iH7WM".to_string();
        // agency_client.my_vk = "1pBNDeG2oPEK44zRMEvKn8GbQQpduGVu3QHExBHEPvR".to_string();

        // connect with some default vcx mediator
        // let config_provision_agent = AgentProvisionConfig {
        //     agency_did: AGENCY_DID.to_string(),
        //     agency_verkey: AGENCY_VERKEY.to_string(),
        //     agency_endpoint: "https://ariesvcx.agency.staging.absa.id".to_string(),
        //     agent_seed: None,
        // };
        // provision_cloud_agent(&mut agency_client, &indy_profile, &config_provision_agent).await;

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

        for (msg_id, _) in msgs {
            conn.update_message_status(msg_id, &agency_client).await.unwrap();
        }
    }

    async fn get_first_connection_msg(conn: &Connection, agency_client: &AgencyClient) -> (String, A2AMessage) {
        let msgs = conn.get_messages_noauth(&agency_client).await.unwrap();
        let msgs = msgs.iter().collect::<Vec<(&String, &A2AMessage)>>();

        let x = msgs.first().expect("no msgs").to_owned();

        (x.0.to_owned(), x.1.to_owned())
    }

    #[tokio::test]
    async fn cred_flow() {
        let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"7ePNQN9QV8NTym41ct724a\",\"pw_vk\":\"4d3oT5pBkXx6SVMQYAMPZp6hmVTAbozRUcq7KZWMAmYY\",\"agent_did\":\"Qu1CWMtxGrmizu48bphT8E\",\"agent_vk\":\"E2RJRmmYwipWgXpHbYap1Dc9Dh6qAxDYNuRMjwAEJcJU\"},\"state\":{\"Invitee\":{\"Responded\":{\"response\":{\"@id\":\"a3309155-4de6-4c4d-8206-4b4b099b4b2c\",\"~thread\":{\"thid\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\",\"sender_order\":0,\"received_orders\":{}},\"connection~sig\":{\"@type\":\"did:sov:BzCbsNYhMrjHiqZDTUASHg/signature/1.0/ed25519Sha512_single\",\"signature\":\"TY-024HPg8eXOMrettwUXF3MhRgOkeZB3htBKx4Yyp9PB0JS0_05BJGj93InUg0PrFHOeiLn1z8K9Ojhp4RGDw==\",\"sig_data\":\"AAAAAGMxSf97IkRJRCI6ICIybVRHSmRoMTdKSENBSE10dDRkRndhIiwgIkRJRERvYyI6IHsiQGNvbnRleHQiOiAiaHR0cHM6Ly93M2lkLm9yZy9kaWQvdjEiLCAiaWQiOiAiZGlkOnNvdjoybVRHSmRoMTdKSENBSE10dDRkRndhIiwgInB1YmxpY0tleSI6IFt7ImlkIjogImRpZDpzb3Y6Mm1UR0pkaDE3SkhDQUhNdHQ0ZEZ3YSMxIiwgInR5cGUiOiAiRWQyNTUxOVZlcmlmaWNhdGlvbktleTIwMTgiLCAiY29udHJvbGxlciI6ICJkaWQ6c292OjJtVEdKZGgxN0pIQ0FITXR0NGRGd2EiLCAicHVibGljS2V5QmFzZTU4IjogInhxamdBS2U3VXpNMkpUd0RNRjV5OWRlWGNLcXo4SGNaTG02OENmaFJVcGsifV0sICJhdXRoZW50aWNhdGlvbiI6IFt7InR5cGUiOiAiRWQyNTUxOVNpZ25hdHVyZUF1dGhlbnRpY2F0aW9uMjAxOCIsICJwdWJsaWNLZXkiOiAiZGlkOnNvdjoybVRHSmRoMTdKSENBSE10dDRkRndhIzEifV0sICJzZXJ2aWNlIjogW3siaWQiOiAiZGlkOnNvdjoybVRHSmRoMTdKSENBSE10dDRkRndhO2luZHkiLCAidHlwZSI6ICJJbmR5QWdlbnQiLCAicHJpb3JpdHkiOiAwLCAicmVjaXBpZW50S2V5cyI6IFsieHFqZ0FLZTdVek0ySlR3RE1GNXk5ZGVYY0txejhIY1pMbTY4Q2ZoUlVwayJdLCAic2VydmljZUVuZHBvaW50IjogImh0dHA6Ly8xYWNhLTEyNS0yNTMtMTYtMTY0Lm5ncm9rLmlvIn1dfX0=\",\"signer\":\"ACaJKSuXyPNo2Skwz57DNfDhu5FpuyR3gPoAxUQUQ3Gn\"}},\"request\":{\"@id\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\",\"label\":\"1\",\"connection\":{\"DID\":\"7ePNQN9QV8NTym41ct724a\",\"DIDDoc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"7ePNQN9QV8NTym41ct724a\",\"publicKey\":[{\"id\":\"7ePNQN9QV8NTym41ct724a#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"7ePNQN9QV8NTym41ct724a\",\"publicKeyBase58\":\"4d3oT5pBkXx6SVMQYAMPZp6hmVTAbozRUcq7KZWMAmYY\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"7ePNQN9QV8NTym41ct724a#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"4d3oT5pBkXx6SVMQYAMPZp6hmVTAbozRUcq7KZWMAmYY\"],\"routingKeys\":[\"E2RJRmmYwipWgXpHbYap1Dc9Dh6qAxDYNuRMjwAEJcJU\",\"Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR\"],\"serviceEndpoint\":\"https://ariesvcx.agency.staging.absa.id/agency/msg\"}]}},\"~thread\":{\"thid\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\",\"sender_order\":0,\"received_orders\":{}},\"~timing\":{\"out_time\":\"2022-09-26T06:43:11.142Z\"}},\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160\",\"publicKey\":[{\"id\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160\",\"publicKeyBase58\":\"ACaJKSuXyPNo2Skwz57DNfDhu5FpuyR3gPoAxUQUQ3Gn\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"ACaJKSuXyPNo2Skwz57DNfDhu5FpuyR3gPoAxUQUQ3Gn\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://1aca-125-253-16-164.ngrok.io\"}]}}}},\"source_id\":\"1\",\"thread_id\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\"}";

        let conn: Connection = Connection::from_string(conn_ser).unwrap();

        let indy_handle = open_default_indy_handle().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile.clone());

        Arc::clone(&profile).inject_anoncreds().prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).await.ok();
        global::pool::open_main_pool(&PoolConfig { genesis_path: settings::DEFAULT_GENESIS_PATH.to_string(), pool_name: None, pool_config: None }).await.unwrap();

        let agency_client = open_default_agency_client(&profile);

        // clear_connection_messages(&conn, &agency_client).await;

        let (msg_id, message) = get_first_connection_msg(&conn, &agency_client).await;

        println!("MESSAGE!: {:?}", message);
         
        let offer: CredentialOffer = match message {
            A2AMessage::CredentialOffer(m) => m.to_owned(),
            _ => panic!("aaaa")
        };

        let mut holder = Holder::create_from_offer("idk", offer).unwrap();

        holder.send_request(&profile, conn.pairwise_info().pw_did.to_string(), conn.send_message_closure(&profile).unwrap()).await.unwrap();

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
            _ => panic!("aaaa")
        };

        holder.step(&profile, CredentialIssuanceAction::Credential(issaunce_msg), Some(conn.send_message_closure(&profile).unwrap())).await.unwrap();

        println!("state; {:?}", holder.get_state());
        println!("cred; {:?}", holder.get_credential());
        println!("whole; {:?}", holder);

        // remove msg
        conn.update_message_status(&msg_id, &agency_client).await.unwrap();

        ()
    }

    #[tokio::test]
    async fn accept_cred() {
         let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"7ePNQN9QV8NTym41ct724a\",\"pw_vk\":\"4d3oT5pBkXx6SVMQYAMPZp6hmVTAbozRUcq7KZWMAmYY\",\"agent_did\":\"Qu1CWMtxGrmizu48bphT8E\",\"agent_vk\":\"E2RJRmmYwipWgXpHbYap1Dc9Dh6qAxDYNuRMjwAEJcJU\"},\"state\":{\"Invitee\":{\"Responded\":{\"response\":{\"@id\":\"a3309155-4de6-4c4d-8206-4b4b099b4b2c\",\"~thread\":{\"thid\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\",\"sender_order\":0,\"received_orders\":{}},\"connection~sig\":{\"@type\":\"did:sov:BzCbsNYhMrjHiqZDTUASHg/signature/1.0/ed25519Sha512_single\",\"signature\":\"TY-024HPg8eXOMrettwUXF3MhRgOkeZB3htBKx4Yyp9PB0JS0_05BJGj93InUg0PrFHOeiLn1z8K9Ojhp4RGDw==\",\"sig_data\":\"AAAAAGMxSf97IkRJRCI6ICIybVRHSmRoMTdKSENBSE10dDRkRndhIiwgIkRJRERvYyI6IHsiQGNvbnRleHQiOiAiaHR0cHM6Ly93M2lkLm9yZy9kaWQvdjEiLCAiaWQiOiAiZGlkOnNvdjoybVRHSmRoMTdKSENBSE10dDRkRndhIiwgInB1YmxpY0tleSI6IFt7ImlkIjogImRpZDpzb3Y6Mm1UR0pkaDE3SkhDQUhNdHQ0ZEZ3YSMxIiwgInR5cGUiOiAiRWQyNTUxOVZlcmlmaWNhdGlvbktleTIwMTgiLCAiY29udHJvbGxlciI6ICJkaWQ6c292OjJtVEdKZGgxN0pIQ0FITXR0NGRGd2EiLCAicHVibGljS2V5QmFzZTU4IjogInhxamdBS2U3VXpNMkpUd0RNRjV5OWRlWGNLcXo4SGNaTG02OENmaFJVcGsifV0sICJhdXRoZW50aWNhdGlvbiI6IFt7InR5cGUiOiAiRWQyNTUxOVNpZ25hdHVyZUF1dGhlbnRpY2F0aW9uMjAxOCIsICJwdWJsaWNLZXkiOiAiZGlkOnNvdjoybVRHSmRoMTdKSENBSE10dDRkRndhIzEifV0sICJzZXJ2aWNlIjogW3siaWQiOiAiZGlkOnNvdjoybVRHSmRoMTdKSENBSE10dDRkRndhO2luZHkiLCAidHlwZSI6ICJJbmR5QWdlbnQiLCAicHJpb3JpdHkiOiAwLCAicmVjaXBpZW50S2V5cyI6IFsieHFqZ0FLZTdVek0ySlR3RE1GNXk5ZGVYY0txejhIY1pMbTY4Q2ZoUlVwayJdLCAic2VydmljZUVuZHBvaW50IjogImh0dHA6Ly8xYWNhLTEyNS0yNTMtMTYtMTY0Lm5ncm9rLmlvIn1dfX0=\",\"signer\":\"ACaJKSuXyPNo2Skwz57DNfDhu5FpuyR3gPoAxUQUQ3Gn\"}},\"request\":{\"@id\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\",\"label\":\"1\",\"connection\":{\"DID\":\"7ePNQN9QV8NTym41ct724a\",\"DIDDoc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"7ePNQN9QV8NTym41ct724a\",\"publicKey\":[{\"id\":\"7ePNQN9QV8NTym41ct724a#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"7ePNQN9QV8NTym41ct724a\",\"publicKeyBase58\":\"4d3oT5pBkXx6SVMQYAMPZp6hmVTAbozRUcq7KZWMAmYY\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"7ePNQN9QV8NTym41ct724a#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"4d3oT5pBkXx6SVMQYAMPZp6hmVTAbozRUcq7KZWMAmYY\"],\"routingKeys\":[\"E2RJRmmYwipWgXpHbYap1Dc9Dh6qAxDYNuRMjwAEJcJU\",\"Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR\"],\"serviceEndpoint\":\"https://ariesvcx.agency.staging.absa.id/agency/msg\"}]}},\"~thread\":{\"thid\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\",\"sender_order\":0,\"received_orders\":{}},\"~timing\":{\"out_time\":\"2022-09-26T06:43:11.142Z\"}},\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160\",\"publicKey\":[{\"id\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160\",\"publicKeyBase58\":\"ACaJKSuXyPNo2Skwz57DNfDhu5FpuyR3gPoAxUQUQ3Gn\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"c0bb7b2b-eaac-43bc-add3-ba6ccc7b9160#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"ACaJKSuXyPNo2Skwz57DNfDhu5FpuyR3gPoAxUQUQ3Gn\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://1aca-125-253-16-164.ngrok.io\"}]}}}},\"source_id\":\"1\",\"thread_id\":\"ee06106d-4d2c-46d0-8996-c331117bb06b\"}";

        let conn: Connection = Connection::from_string(conn_ser).unwrap();

        let indy_handle = open_default_indy_handle().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile.clone());

        Arc::clone(&profile).inject_anoncreds().prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).await.ok();
        global::pool::open_main_pool(&PoolConfig { genesis_path: settings::DEFAULT_GENESIS_PATH.to_string(), pool_name: None, pool_config: None }).await.unwrap();

        let agency_client = open_default_agency_client(&profile);

        let msgs = conn.get_messages_noauth(&agency_client).await.unwrap();
        let msgs = msgs.iter().collect::<Vec<(&String, &A2AMessage)>>();
        let (msg_id, message) = msgs.first().expect("bruh").clone();

        println!("MESSAGE! ({:?}): {:?}", msg_id, message);

         let issaunce_msg: Credential = match message {
            A2AMessage::Credential(m) => m.to_owned(),
            _ => panic!("aaaa")
        };

        let mut holder = Holder::create("1").unwrap();
        holder.step(&profile, CredentialIssuanceAction::Credential(issaunce_msg), Some(conn.send_message_closure(&profile).unwrap())).await.unwrap();

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
