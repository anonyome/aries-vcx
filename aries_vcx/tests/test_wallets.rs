#[cfg(test)]
#[cfg(feature = "wallet_tests")]
mod integration_tests {
    use std::{sync::Arc, thread, time::Duration};

    use agency_client::{agency_client::AgencyClient, configuration::AgentProvisionConfig};
    use aries_vcx::messages::a2a::A2AMessage;
    use aries_vcx::{
        core::profile::{indy_profile::IndySdkProfile, profile::Profile},
        global::settings,
        handlers::connection::connection::Connection,
        libindy::utils::wallet::{create_and_open_wallet, WalletConfig},
        messages::connection::invite::Invitation,
        utils::devsetup::{AGENCY_DID, AGENCY_VERKEY},
        wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    };
    use indy_sys::WalletHandle;

    use crate::integration_tests::helper::provision_cloud_agent;

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

    #[tokio::test]
    async fn test_connection() {
        let indy_handle = open_default_indy_handle().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile.clone());

        let mut agency_client = AgencyClient::new();

        let invitation = helper::url_to_invitation("https://trinsic.studio/link/?d_m=eyJsYWJlbCI6IkhlbGxvV29ybGQiLCJpbWFnZVVybCI6Imh0dHBzOi8vdHJpbnNpY2FwaWFzc2V0cy5henVyZWVkZ2UubmV0L2ZpbGVzLzc3NTM3ZGU0LWU0YTYtNDUwMS05OTJkLTQ2NGE4MmRiOTFkNl9iYjQwZDUzNC1jNDgyLTQ0YWYtOGQwYS00NmQ5ODQwNzFkZGEucG5nIiwic2VydmljZUVuZHBvaW50IjoiaHR0cHM6Ly9hcGkucG9ydGFsLnN0cmVldGNyZWQuaWQvYWdlbnQva1hmVkhkd2s4MUZKeE40b2lQUHpnaTc2blhUTUY3YzkiLCJyb3V0aW5nS2V5cyI6WyI2cGVLYVV4ZG9yTlVtRVl5Q2JYbXRKWXVhcG1vcDVQUUoyMVh6ZGcxWk1YdCJdLCJyZWNpcGllbnRLZXlzIjpbImY2ZW85RHJGVGtIbmVhOWtMRUZ2cDd2Skhicll1RFNFRW9RN05UdVU4cW0iXSwiQGlkIjoiYWI5NmE3YWItYjg3OC00ZWI1LWEwNjgtMTNlODk3MGRkY2YzIiwiQHR5cGUiOiJkaWQ6c292OkJ6Q2JzTlloTXJqSGlxWkRUVUFTSGc7c3BlYy9jb25uZWN0aW9ucy8xLjAvaW52aXRhdGlvbiJ9&orig=https://trinsic.studio/url/cfba1059-6d34-420b-84a4-a6e0d6eec95b");
        // invitation.service_endpoint = "http://localhost:8200".to_string();
        let invitation = Invitation::Pairwise(invitation);

        // connect with some default vcx mediator
        let config_provision_agent = AgentProvisionConfig {
            agency_did: AGENCY_DID.to_string(),
            agency_verkey: AGENCY_VERKEY.to_string(),
            agency_endpoint: "https://ariesvcx.agency.staging.absa.id".to_string(),
            agent_seed: None,
        };
        provision_cloud_agent(&mut agency_client, &indy_profile, &config_provision_agent).await;

        println!("agency client; {:?}", agency_client);

        // receive and accept invite
        // note that trinsic doesn't understand the ACK, so turn it off
        let autohop = false;
        let mut conn = Connection::create_with_invite("7", &profile, &agency_client, invitation, autohop)
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
        let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"FNknW5yxE9h4PZSEXzz9DV\",\"pw_vk\":\"8qRduDFo2rkSWUbrQGWEMHidRsAzgYNxud1grCczxLjs\",\"agent_did\":\"F71Z3rKD2buJFfdHBaqXCZ\",\"agent_vk\":\"8gqtqdAaZi2JHZA19prZs8aZghWSFRXCSUgTYCm8rnAZ\"},\"state\":{\"Invitee\":{\"Responded\":{\"response\":{\"@id\":\"788c6ccc-3862-49e8-aef5-fc10b6fceee1\",\"~thread\":{\"thid\":\"18dc73d9-ac5b-4b39-8a5d-26f4ba6f13c8\",\"sender_order\":0,\"received_orders\":{}},\"connection~sig\":{\"@type\":\"did:sov:BzCbsNYhMrjHiqZDTUASHg/signature/1.0/ed25519Sha512_single\",\"signature\":\"JVGCSmmURT53ZPoI49A7tz56FkWV3gbbWUv4TgA5u3OSQVgF2bAjwbBWlB_1znwsc5ao-rRVlUDZyh-llQdfDg==\",\"sig_data\":\"ESstYwAAAAB7IkRJRCI6IjlCeWRLNWFkY2lUTlBQQnNFd3k2VXYiLCJESUREb2MiOnsiQGNvbnRleHQiOiJodHRwczovL3czaWQub3JnL2RpZC92MSIsImlkIjoiOUJ5ZEs1YWRjaVROUFBCc0V3eTZVdiIsInB1YmxpY0tleSI6W3siaWQiOiI5QnlkSzVhZGNpVE5QUEJzRXd5NlV2I2tleXMtMSIsInR5cGUiOiJFZDI1NTE5VmVyaWZpY2F0aW9uS2V5MjAxOCIsImNvbnRyb2xsZXIiOiI5QnlkSzVhZGNpVE5QUEJzRXd5NlV2IiwicHVibGljS2V5QmFzZTU4IjoiNVRzd055WFJXOEFpRzZYVFpYYWZGeE54OXdZMVZNVkIybW9udEZkMmlFQ3QifV0sInNlcnZpY2UiOlt7ImlkIjoiOUJ5ZEs1YWRjaVROUFBCc0V3eTZVdjtpbmR5IiwidHlwZSI6IkluZHlBZ2VudCIsInJlY2lwaWVudEtleXMiOlsiNVRzd055WFJXOEFpRzZYVFpYYWZGeE54OXdZMVZNVkIybW9udEZkMmlFQ3QiXSwicm91dGluZ0tleXMiOlsiNnBlS2FVeGRvck5VbUVZeUNiWG10Sll1YXBtb3A1UFFKMjFYemRnMVpNWHQiXSwic2VydmljZUVuZHBvaW50IjoiaHR0cHM6Ly9hcGkucG9ydGFsLnN0cmVldGNyZWQuaWQvYWdlbnQva1hmVkhkd2s4MUZKeE40b2lQUHpnaTc2blhUTUY3YzkifV19fQ==\",\"signer\":\"HG1oejRBMqSDWZ2HZaDcFhuGnwPFUsmq34gA9LSPimJY\"}},\"request\":{\"@id\":\"18dc73d9-ac5b-4b39-8a5d-26f4ba6f13c8\",\"label\":\"6\",\"connection\":{\"DID\":\"FNknW5yxE9h4PZSEXzz9DV\",\"DIDDoc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"FNknW5yxE9h4PZSEXzz9DV\",\"publicKey\":[{\"id\":\"FNknW5yxE9h4PZSEXzz9DV#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"FNknW5yxE9h4PZSEXzz9DV\",\"publicKeyBase58\":\"8qRduDFo2rkSWUbrQGWEMHidRsAzgYNxud1grCczxLjs\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"FNknW5yxE9h4PZSEXzz9DV#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"8qRduDFo2rkSWUbrQGWEMHidRsAzgYNxud1grCczxLjs\"],\"routingKeys\":[\"8gqtqdAaZi2JHZA19prZs8aZghWSFRXCSUgTYCm8rnAZ\",\"Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR\"],\"serviceEndpoint\":\"https://ariesvcx.agency.staging.absa.id/agency/msg\"}]}},\"~thread\":{\"thid\":\"18dc73d9-ac5b-4b39-8a5d-26f4ba6f13c8\",\"sender_order\":0,\"received_orders\":{}},\"~timing\":{\"out_time\":\"2022-09-23T03:42:07.455Z\"}},\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"59fe3bb2-8842-4861-9807-bc2a2c809fa6\",\"publicKey\":[{\"id\":\"59fe3bb2-8842-4861-9807-bc2a2c809fa6#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"59fe3bb2-8842-4861-9807-bc2a2c809fa6\",\"publicKeyBase58\":\"HG1oejRBMqSDWZ2HZaDcFhuGnwPFUsmq34gA9LSPimJY\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"59fe3bb2-8842-4861-9807-bc2a2c809fa6#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"HG1oejRBMqSDWZ2HZaDcFhuGnwPFUsmq34gA9LSPimJY\"],\"routingKeys\":[\"6peKaUxdorNUmEYyCbXmtJYuapmop5PQJ21Xzdg1ZMXt\"],\"serviceEndpoint\":\"https://api.portal.streetcred.id/agent/kXfVHdwk81FJxN4oiPPzgi76nXTMF7c9\"}]}}}},\"source_id\":\"6\",\"thread_id\":\"18dc73d9-ac5b-4b39-8a5d-26f4ba6f13c8\"}";

        let conn: Connection = Connection::from_string(conn_ser).unwrap();

        let indy_handle = open_default_indy_handle().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile.clone());

        let mut agency_client = AgencyClient::new();

        agency_client.set_wallet_handle(indy_handle);
        agency_client.agency_url = "https://ariesvcx.agency.staging.absa.id".to_string();
        agency_client.agency_did = AGENCY_DID.to_string();
        agency_client.agency_vk = AGENCY_VERKEY.to_string();
        agency_client.agent_pwdid = "XVciZMJAYfn4i5fFNHZ1SC".to_string();
        agency_client.agent_vk = "HcxS4fnkcUy9jfZ5R5v88Rngw3npSLUv17pthXNNCvnz".to_string();
        agency_client.my_pwdid = "12VZYR1AarNNQYAa8iH7WM".to_string();
        agency_client.my_vk = "1pBNDeG2oPEK44zRMEvKn8GbQQpduGVu3QHExBHEPvR".to_string();

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

        conn.update_message_status("fd4f97ca-7894-4d8e-a857-0506e3c551b0", &agency_client)
            .await
            .unwrap();

        let msgs = conn.get_messages_noauth(&agency_client).await.unwrap();
        let message = msgs
            .iter()
            .map(|i| i.1.to_owned())
            .collect::<Vec<A2AMessage>>()
            .first()
            .map(|a| a.to_owned());

        println!("MESSAGE!: {:?}", message);

        // Holder::create_from_offer(source_id, credential_offer)

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
                    profile.indy_handle,
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
