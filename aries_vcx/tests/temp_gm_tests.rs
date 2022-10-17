#[cfg(test)]
#[cfg(feature = "temp_gm_tests")]
mod integration_tests {
    use aries_vcx::core::profile::modular_wallet_profile::{LedgerPoolConfig, ModularWalletProfile};
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
    async fn establish_connection() {
        let indy_handle = setup::open_default_indy_handle().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile.clone());

        let agency_client = setup::open_default_agency_client(&profile);

        let invitation = helper::url_to_invitation("http://cloudagent.gmulhearne.di-team.dev.sudoplatform.com:8200?c_i=eyJAdHlwZSI6ICJkaWQ6c292OkJ6Q2JzTlloTXJqSGlxWkRUVUFTSGc7c3BlYy9jb25uZWN0aW9ucy8xLjAvaW52aXRhdGlvbiIsICJAaWQiOiAiN2Y3NTRhY2UtNDZhZC00NzExLTlkMWEtZmRjY2UxN2FmOTJmIiwgInJlY2lwaWVudEtleXMiOiBbIjZVb25kTTJTam5XVkRLeGFac2I5d0FUWkNYRW9ZRHNLcGdUdDc4NmRTV29iIl0sICJzZXJ2aWNlRW5kcG9pbnQiOiAiaHR0cDovL2Nsb3VkYWdlbnQuZ211bGhlYXJuZS5kaS10ZWFtLmRldi5zdWRvcGxhdGZvcm0uY29tOjgyMDAiLCAibGFiZWwiOiAiZ211bGhlYXJuZSJ9");
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

    #[tokio::test]
    async fn clear_messages() {
        let (conn, _, _, _, agency_client) = setup::setup_with_existing_conn().await;
        helper::clear_connection_messages(&conn, &agency_client).await;
    }

    #[tokio::test]
    async fn cred_issuance_flow() {
        let (conn, indy_handle, mod_profile, indy_profile, agency_client) = setup::setup_with_existing_conn().await;

        // choose which profile to use
        let profile = mod_profile;

        println!(
            "{}",
            profile.clone().inject_anoncreds().prover_get_credentials(None).await.unwrap()
            // indyrs::anoncreds::prover_get_credentials(indy_handle, None)
            //     .await
            //     .unwrap()
        );

        let (msg_id, message) = helper::get_first_connection_msg(&conn, &profile, &agency_client).await;

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

        let (msg_id, message) = helper::get_first_connection_msg(&conn, &profile, &agency_client).await;
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
    async fn proof_presentation_flow() {
        let (conn, _, mod_profile, indy_profile, agency_client) = setup::setup_with_existing_conn().await;

        // choose which profile to use
        let profile = mod_profile;

        println!(
            "{}",
            Arc::clone(&profile)
                .inject_anoncreds()
                .prover_get_credentials(None)
                .await
                .unwrap()
        );

        let (msg_id, message) = helper::get_first_connection_msg(&conn, &profile, &agency_client).await;

        let pres_req: PresentationRequest = match message {
            A2AMessage::PresentationRequest(m) => m.to_owned(),
            _ => panic!("aaaa"),
        };

        let mut prover = Prover::create_from_request("1", pres_req).unwrap();

        // conn.update_message_status(&msg_id, &agency_client).await.unwrap();

        let creds = prover.retrieve_credentials(&profile).await.unwrap();
        println!("creds; {}", creds);

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

        let (msg_id, message) = helper::get_first_connection_msg(&conn, &profile, &agency_client).await;
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

    mod setup {
        use std::sync::Arc;

        use agency_client::agency_client::AgencyClient;
        use aries_vcx::{
            core::profile::{
                indy_profile::IndySdkProfile,
                modular_wallet_profile::{LedgerPoolConfig, ModularWalletProfile},
                profile::Profile,
            },
            global::{self, settings},
            handlers::connection::connection::Connection,
            libindy::utils::{pool::PoolConfig, wallet::WalletConfig},
            plugins::wallet::agency_client_wallet::ToBaseAgencyClientWallet,
            utils::devsetup::{AGENCY_DID, AGENCY_VERKEY},
        };
        use indy_sys::WalletHandle;

        pub async fn open_default_indy_handle() -> WalletHandle {
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

        pub fn open_default_agency_client(profile: &Arc<dyn Profile>) -> AgencyClient {
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

        pub async fn setup_with_existing_conn() -> (
            Connection,
            WalletHandle,
            Arc<dyn Profile>,
            Arc<dyn Profile>,
            AgencyClient,
        ) {
            // aca (VCX1)
            let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"2qy79eGUTrtmdpjjNAGaZD\",\"pw_vk\":\"21JT2GgDbh84DYiG6HgNUM7d6HXtM9sBm8sXaF5kPJ3a\",\"agent_did\":\"MoL5ZJoYcApQCPerrbd2kb\",\"agent_vk\":\"CLVSHaD6oeHxA6sDe8CJ5HKbDoMnjXwJMF9XnJ2QTTA7\"},\"state\":{\"Invitee\":{\"Responded\":{\"response\":{\"@id\":\"557a1e9d-c49a-4fa0-a183-06acc1a38a06\",\"~thread\":{\"thid\":\"7f754ace-46ad-4711-9d1a-fdcce17af92f\",\"sender_order\":0,\"received_orders\":{}},\"connection~sig\":{\"@type\":\"did:sov:BzCbsNYhMrjHiqZDTUASHg/signature/1.0/ed25519Sha512_single\",\"signature\":\"JWwAWCm1pvJFSS1LRM4GVJwQ3cvStwbBO57-mYXtQJImgKpjHP31l66856dZLFqwaDT71HnQm3UdZJfY9tX9BA==\",\"sig_data\":\"AAAAAGNExZt7IkRJRCI6ICJXZ2p2bnk3a3VNUzZrNjQyTTRCYVZyIiwgIkRJRERvYyI6IHsiQGNvbnRleHQiOiAiaHR0cHM6Ly93M2lkLm9yZy9kaWQvdjEiLCAiaWQiOiAiZGlkOnNvdjpXZ2p2bnk3a3VNUzZrNjQyTTRCYVZyIiwgInB1YmxpY0tleSI6IFt7ImlkIjogImRpZDpzb3Y6V2dqdm55N2t1TVM2azY0Mk00QmFWciMxIiwgInR5cGUiOiAiRWQyNTUxOVZlcmlmaWNhdGlvbktleTIwMTgiLCAiY29udHJvbGxlciI6ICJkaWQ6c292OldnanZueTdrdU1TNms2NDJNNEJhVnIiLCAicHVibGljS2V5QmFzZTU4IjogIkhCUWJuS2o5bzFHaFZHQjFTQ2drUEFkamJ3YVpSRHA0MWdYRWdtQ3hrYTI1In1dLCAiYXV0aGVudGljYXRpb24iOiBbeyJ0eXBlIjogIkVkMjU1MTlTaWduYXR1cmVBdXRoZW50aWNhdGlvbjIwMTgiLCAicHVibGljS2V5IjogImRpZDpzb3Y6V2dqdm55N2t1TVM2azY0Mk00QmFWciMxIn1dLCAic2VydmljZSI6IFt7ImlkIjogImRpZDpzb3Y6V2dqdm55N2t1TVM2azY0Mk00QmFWcjtpbmR5IiwgInR5cGUiOiAiSW5keUFnZW50IiwgInByaW9yaXR5IjogMCwgInJlY2lwaWVudEtleXMiOiBbIkhCUWJuS2o5bzFHaFZHQjFTQ2drUEFkamJ3YVpSRHA0MWdYRWdtQ3hrYTI1Il0sICJzZXJ2aWNlRW5kcG9pbnQiOiAiaHR0cDovL2Nsb3VkYWdlbnQuZ211bGhlYXJuZS5kaS10ZWFtLmRldi5zdWRvcGxhdGZvcm0uY29tOjgyMDAifV19fQ==\",\"signer\":\"6UondM2SjnWVDKxaZsb9wATZCXEoYDsKpgTt786dSWob\"}},\"request\":{\"@id\":\"db914919-e395-4e85-a90c-f95e86acaeb0\",\"label\":\"69\",\"connection\":{\"DID\":\"2qy79eGUTrtmdpjjNAGaZD\",\"DIDDoc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"2qy79eGUTrtmdpjjNAGaZD\",\"publicKey\":[{\"id\":\"2qy79eGUTrtmdpjjNAGaZD#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"2qy79eGUTrtmdpjjNAGaZD\",\"publicKeyBase58\":\"21JT2GgDbh84DYiG6HgNUM7d6HXtM9sBm8sXaF5kPJ3a\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"2qy79eGUTrtmdpjjNAGaZD#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"21JT2GgDbh84DYiG6HgNUM7d6HXtM9sBm8sXaF5kPJ3a\"],\"routingKeys\":[\"CLVSHaD6oeHxA6sDe8CJ5HKbDoMnjXwJMF9XnJ2QTTA7\",\"Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR\"],\"serviceEndpoint\":\"https://ariesvcx.agency.staging.absa.id/agency/msg\"}]}},\"~thread\":{\"thid\":\"7f754ace-46ad-4711-9d1a-fdcce17af92f\",\"sender_order\":0,\"received_orders\":{}},\"~timing\":{\"out_time\":\"2022-10-11T01:23:38.363Z\"}},\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"7f754ace-46ad-4711-9d1a-fdcce17af92f\",\"publicKey\":[{\"id\":\"7f754ace-46ad-4711-9d1a-fdcce17af92f#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"7f754ace-46ad-4711-9d1a-fdcce17af92f\",\"publicKeyBase58\":\"6UondM2SjnWVDKxaZsb9wATZCXEoYDsKpgTt786dSWob\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"7f754ace-46ad-4711-9d1a-fdcce17af92f#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"6UondM2SjnWVDKxaZsb9wATZCXEoYDsKpgTt786dSWob\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://cloudagent.gmulhearne.di-team.dev.sudoplatform.com:8200\"}]}}}},\"source_id\":\"69\",\"thread_id\":\"7f754ace-46ad-4711-9d1a-fdcce17af92f\"}";
            // trin (69)
            // let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"ThFAd68nv6qFcQVhm2LZEp\",\"pw_vk\":\"FYr43xb7eDfhJ8KYgur4BiaDUbF5a7hB1yeFD5Dp48j6\",\"agent_did\":\"VHJzeRk9iufdVcHC4kQSH3\",\"agent_vk\":\"GR2SYM4VGDSnuexUqDRkQHrzJCGnTMK16nx9nsjW6EQK\"},\"state\":{\"Invitee\":{\"Responded\":{\"response\":{\"@id\":\"9c936654-255b-4bba-8838-476a5034f7b9\",\"~thread\":{\"thid\":\"a137601f-bcf6-48bf-ad2f-fd4afd315978\",\"sender_order\":0,\"received_orders\":{}},\"connection~sig\":{\"@type\":\"did:sov:BzCbsNYhMrjHiqZDTUASHg/signature/1.0/ed25519Sha512_single\",\"signature\":\"B2_QxoAfcMo6Dh6-CUbsnA624sokPzsrN3qYWCLSp1fNPH0_yOLe4_w7lAeHYiVZVV-Gii0lP4UMfjN7YcvyDw==\",\"sig_data\":\"i0w2YwAAAAB7IkRJRCI6IlRTV1FOUHM5SlFaU21MY3VGc3JqVlQiLCJESUREb2MiOnsiQGNvbnRleHQiOiJodHRwczovL3czaWQub3JnL2RpZC92MSIsImlkIjoiVFNXUU5QczlKUVpTbUxjdUZzcmpWVCIsInB1YmxpY0tleSI6W3siaWQiOiJUU1dRTlBzOUpRWlNtTGN1RnNyalZUI2tleXMtMSIsInR5cGUiOiJFZDI1NTE5VmVyaWZpY2F0aW9uS2V5MjAxOCIsImNvbnRyb2xsZXIiOiJUU1dRTlBzOUpRWlNtTGN1RnNyalZUIiwicHVibGljS2V5QmFzZTU4IjoiRlFwQkpyZW5OOTlWTFNSWmYxZ0VMUzN1dWY0WjMxM0VkWVR0alFnNVZmYlgifV0sInNlcnZpY2UiOlt7ImlkIjoiVFNXUU5QczlKUVpTbUxjdUZzcmpWVDtpbmR5IiwidHlwZSI6IkluZHlBZ2VudCIsInJlY2lwaWVudEtleXMiOlsiRlFwQkpyZW5OOTlWTFNSWmYxZ0VMUzN1dWY0WjMxM0VkWVR0alFnNVZmYlgiXSwicm91dGluZ0tleXMiOlsiNnBlS2FVeGRvck5VbUVZeUNiWG10Sll1YXBtb3A1UFFKMjFYemRnMVpNWHQiXSwic2VydmljZUVuZHBvaW50IjoiaHR0cHM6Ly9hcGkucG9ydGFsLnN0cmVldGNyZWQuaWQvYWdlbnQva1hmVkhkd2s4MUZKeE40b2lQUHpnaTc2blhUTUY3YzkifV19fQ==\",\"signer\":\"jHhBiCXniMNe2SAYYZA466M5eoM53sR4FxQ1dNhG2rZ\"}},\"request\":{\"@id\":\"a137601f-bcf6-48bf-ad2f-fd4afd315978\",\"label\":\"69\",\"connection\":{\"DID\":\"ThFAd68nv6qFcQVhm2LZEp\",\"DIDDoc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"ThFAd68nv6qFcQVhm2LZEp\",\"publicKey\":[{\"id\":\"ThFAd68nv6qFcQVhm2LZEp#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"ThFAd68nv6qFcQVhm2LZEp\",\"publicKeyBase58\":\"FYr43xb7eDfhJ8KYgur4BiaDUbF5a7hB1yeFD5Dp48j6\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"ThFAd68nv6qFcQVhm2LZEp#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"FYr43xb7eDfhJ8KYgur4BiaDUbF5a7hB1yeFD5Dp48j6\"],\"routingKeys\":[\"GR2SYM4VGDSnuexUqDRkQHrzJCGnTMK16nx9nsjW6EQK\",\"Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR\"],\"serviceEndpoint\":\"https://ariesvcx.agency.staging.absa.id/agency/msg\"}]}},\"~thread\":{\"thid\":\"a137601f-bcf6-48bf-ad2f-fd4afd315978\",\"pthid\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715\",\"sender_order\":0,\"received_orders\":{}},\"~timing\":{\"out_time\":\"2022-09-30T01:55:17.700Z\"}},\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715\",\"publicKey\":[{\"id\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715\",\"publicKeyBase58\":\"jHhBiCXniMNe2SAYYZA466M5eoM53sR4FxQ1dNhG2rZ\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"6b6c959b-bdb8-4fac-a8b1-083692dd8715#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"jHhBiCXniMNe2SAYYZA466M5eoM53sR4FxQ1dNhG2rZ\"],\"routingKeys\":[\"6peKaUxdorNUmEYyCbXmtJYuapmop5PQJ21Xzdg1ZMXt\"],\"serviceEndpoint\":\"https://api.portal.streetcred.id/agent/kXfVHdwk81FJxN4oiPPzgi76nXTMF7c9\"}]}}}},\"source_id\":\"69\",\"thread_id\":\"a137601f-bcf6-48bf-ad2f-fd4afd315978\"}";

            let conn: Connection = Connection::from_string(conn_ser).unwrap();

            // set up indy profile
            let indy_handle = open_default_indy_handle().await;
            let indy_profile: Arc<dyn Profile> = Arc::new(IndySdkProfile::new(indy_handle));
            // ------
            //set up modular wallet profile
            let indy_wallet = indy_profile.inject_wallet();
            let ledger_pool_config = LedgerPoolConfig {
            genesis_file_path:
                "/Users/gmulhearne/Documents/dev/platform/di-edge-agent/edge-agent-core/aries-vcx/aries_vcx/genesis.txn"
                    .to_string(),
        };
            let mod_profile: Arc<dyn Profile> =
                Arc::new(ModularWalletProfile::new(indy_wallet, ledger_pool_config).unwrap());
            // ------

            // set up agency client (note hackyness: we only need to do this once for indy_profile bcus mod_profile uses the same wallet)
            let agency_client = open_default_agency_client(&indy_profile);

            // initialization for indy profile
            Arc::clone(&indy_profile)
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
            // ----
            // initialization for modular wallet profile
            Arc::clone(&mod_profile)
                .inject_anoncreds()
                .prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS)
                .await
                .ok();
            // -----

            // configure global TAA
            let precise_time = time::get_time().sec as u64;
            const DAY_SECS: u64 = 60 * 60 * 24;
            aries_vcx::utils::author_agreement::set_txn_author_agreement(
                None,
                Some("1.0".to_string()),
                Some("2f630f02cb1e88d1169db7b4dd0e45943ac1530630d737be5499c8f01c2695b1".to_string()),
                "for_session".to_string(),
                (precise_time / DAY_SECS) * DAY_SECS,
            )
            .unwrap();

            return (conn, indy_handle, mod_profile, indy_profile, agency_client);
        }
    }

    mod helper {

        use std::{fs::File, io::Write, path::Path, sync::Arc};

        use agency_client::{
            agency_client::AgencyClient,
            configuration::{AgencyClientConfig, AgentProvisionConfig},
        };
        use aries_vcx::{
            core::profile::{indy_profile::IndySdkProfile, profile::Profile},
            handlers::connection::connection::Connection,
            messages::{connection::invite::PairwiseInvitation, a2a::A2AMessage},
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

        pub async fn clear_connection_messages(conn: &Connection, agency_client: &AgencyClient) {
            let msgs = conn.get_messages_noauth(&agency_client).await.unwrap();
            let msgs = msgs.iter().collect::<Vec<(&String, &A2AMessage)>>();

            let len = msgs.len();

            for (msg_id, _) in msgs {
                conn.update_message_status(msg_id, &agency_client).await.unwrap();
            }

            println!("Cleared {:?} messages", len);
        }

        pub async fn get_first_connection_msg(
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
