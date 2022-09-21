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
        libindy::utils::{
            crypto::pack_message,
            wallet::{create_and_open_wallet, WalletConfig},
        },
        messages::{
            a2a::MessageId,
            connection::invite::{Invitation, PairwiseInvitation},
        },
        utils::{
            devsetup::{AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY},
            provision::provision_cloud_agent,
        },
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

    #[tokio::test]
    async fn test_connection() {
        
        let indy_handle = open_default_indy_handle().await;

        let indy_profile: Arc<dyn Profile> = Arc::new(IndySdkProfile::new(indy_handle));

        let mut agency_client = AgencyClient::new();

        let invitation = helper::url_to_invitation("https://trinsic.studio/link/?c_i=eyJsYWJlbCI6IkhlbGxvV29ybGQiLCJpbWFnZVVybCI6Imh0dHBzOi8vdHJpbnNpY2FwaWFzc2V0cy5henVyZWVkZ2UubmV0L2ZpbGVzLzc3NTM3ZGU0LWU0YTYtNDUwMS05OTJkLTQ2NGE4MmRiOTFkNl9iYjQwZDUzNC1jNDgyLTQ0YWYtOGQwYS00NmQ5ODQwNzFkZGEucG5nIiwic2VydmljZUVuZHBvaW50IjoiaHR0cHM6Ly9hcGkucG9ydGFsLnN0cmVldGNyZWQuaWQvYWdlbnQva1hmVkhkd2s4MUZKeE40b2lQUHpnaTc2blhUTUY3YzkiLCJyb3V0aW5nS2V5cyI6WyI2cGVLYVV4ZG9yTlVtRVl5Q2JYbXRKWXVhcG1vcDVQUUoyMVh6ZGcxWk1YdCJdLCJyZWNpcGllbnRLZXlzIjpbIkJNVjMyeFpyeFFhUjhkdXo2MUhzRFhXQ3BSUkpBb2loTXk1UmhtYnl5c0ViIl0sIkBpZCI6IjI3NjljNGM4LWZlMjgtNGQxZS04MDQyLWVkZWRhM2QxMzE1NCIsIkB0eXBlIjoiZGlkOnNvdjpCekNic05ZaE1yakhpcVpEVFVBU0hnO3NwZWMvY29ubmVjdGlvbnMvMS4wL2ludml0YXRpb24ifQ%3D%3D&orig=https://trinsic.studio/url/7cf02aa5-032a-4a60-a00f-efcbdbfa8fcf");
        // invitation.service_endpoint = "http://localhost:8200".to_string();
        let invitation = Invitation::Pairwise(invitation);

        // connect with some default vcx mediator
        let config_provision_agent = AgentProvisionConfig {
            agency_did: AGENCY_DID.to_string(),
            agency_verkey: AGENCY_VERKEY.to_string(),
            agency_endpoint: "https://ariesvcx.agency.staging.absa.id".to_string(),
            agent_seed: None,
        };
        provision_cloud_agent(&mut agency_client, indy_handle, &config_provision_agent)
            .await
            .unwrap();

        // receive and accept invite
        let mut conn = Connection::create_with_invite("booboobbb", &indy_profile, &agency_client, invitation, true)
            .await
            .unwrap();
        conn.connect(&indy_profile, &agency_client).await.unwrap();

        println!("{:?}", conn.get_state());

        thread::sleep(Duration::from_millis(5000));

        // // find response and accept
        // conn.find_message_and_update_state(&indy_profile, &agency_client)
        //     .await
        //     .unwrap();

        // fetch response message and input into state update
        let msgs = conn.get_messages_noauth(&agency_client).await.unwrap();
        let response_message = msgs
            .iter()
            .map(|i| i.1.to_owned())
            .collect::<Vec<A2AMessage>>()
            .first()
            .map(|a| a.to_owned());

        println!("RESAPONSE MESGG: {:?}", response_message);
        conn.update_state_with_message(&indy_profile, agency_client, response_message)
            .await
            .unwrap();

        println!("{:?}", conn.get_state());

        conn.send_generic_message(&indy_profile, "hellooooo world, ya ya ya")
            .await
            .unwrap();

        println!("{:?}", conn.to_string().unwrap());

        ()
    }

    #[tokio::test]
    async fn restore_connection() {
        let conn_ser = "{\"version\":\"1.0\",\"data\":{\"pw_did\":\"5XBkvHKXNALBnBQkHgeGQz\",\"pw_vk\":\"3TuCaJ8fSjDFsu9PaXpVkeVB2v3mDXh2gaLujpAEc3Bn\",\"agent_did\":\"YL2TEBpYiNNwiFtVyfUHCX\",\"agent_vk\":\"J5Lk89qiQqCJ448JDco2qyUKLgU3Zrjnk7JcHMTxsZ5D\"},\"state\":{\"Invitee\":{\"Completed\":{\"did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"did:sov:ErVSKCwhT54tqiG2CWJhT5\",\"publicKey\":[{\"id\":\"did:sov:ErVSKCwhT54tqiG2CWJhT5#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"did:sov:ErVSKCwhT54tqiG2CWJhT5\",\"publicKeyBase58\":\"8YvutUX7BAELEJM9c2CDek9BxboMLdfd3JVp2uoyGhHB\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"did:sov:ErVSKCwhT54tqiG2CWJhT5#1\"}],\"service\":[{\"id\":\"did:sov:ErVSKCwhT54tqiG2CWJhT5;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"8YvutUX7BAELEJM9c2CDek9BxboMLdfd3JVp2uoyGhHB\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://9fce-125-253-16-164.ngrok.io\"}]},\"bootstrap_did_doc\":{\"@context\":\"https://w3id.org/did/v1\",\"id\":\"b02e170c-92c1-490d-bd7c-45c7e50c7a74\",\"publicKey\":[{\"id\":\"b02e170c-92c1-490d-bd7c-45c7e50c7a74#1\",\"type\":\"Ed25519VerificationKey2018\",\"controller\":\"b02e170c-92c1-490d-bd7c-45c7e50c7a74\",\"publicKeyBase58\":\"CSuz5H3EWq7py1H7YaGrtQPUyz9Mcn582aH98SR3qZ63\"}],\"authentication\":[{\"type\":\"Ed25519SignatureAuthentication2018\",\"publicKey\":\"b02e170c-92c1-490d-bd7c-45c7e50c7a74#1\"}],\"service\":[{\"id\":\"did:example:123456789abcdefghi;indy\",\"type\":\"IndyAgent\",\"priority\":0,\"recipientKeys\":[\"CSuz5H3EWq7py1H7YaGrtQPUyz9Mcn582aH98SR3qZ63\"],\"routingKeys\":[],\"serviceEndpoint\":\"http://9fce-125-253-16-164.ngrok.io\"}]},\"protocols\":null}}},\"source_id\":\"source_id\",\"thread_id\":\"b02e170c-92c1-490d-bd7c-45c7e50c7a74\"}";

        let conn: Connection = Connection::from_string(conn_ser).unwrap();

        let indy_handle = open_default_indy_handle().await;

        let indy_profile: Arc<dyn Profile> = Arc::new(IndySdkProfile::new(indy_handle));

        conn.send_generic_message(&indy_profile, "hellloooooooooooooooooooooooooo").await.unwrap();

        ()
    }

    mod helper {
        use aries_vcx::messages::connection::invite::PairwiseInvitation;
        use url::Url;

        pub fn url_to_invitation(url: &str) -> PairwiseInvitation {
            let b64_val = Url::parse(url)
                .unwrap()
                .query_pairs()
                .find(|(x, _)| x == "c_i")
                .unwrap()
                .1
                .to_string();

            let v = String::from_utf8(base64::decode_config(&b64_val, base64::URL_SAFE).unwrap()).unwrap();

            serde_json::from_str(&v).unwrap()
        }
    }
}
