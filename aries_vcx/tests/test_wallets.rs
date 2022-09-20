#[cfg(test)]
#[cfg(feature = "wallet_tests")]
mod integration_tests {
    use std::{thread, time::Duration, sync::Arc};

    use agency_client::{agency_client::AgencyClient, configuration::AgentProvisionConfig};
    use aries_vcx::{
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

    #[tokio::test]
    async fn test_connection() {
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

        let wallet = Arc::new(IndySdkWallet::new(indy_handle)) as Arc<dyn BaseWallet>;

        let mut agency_client = AgencyClient::new();

        let mut invitation = helper::url_to_invitation("http://fe0d-125-253-16-164.ngrok.io?c_i=eyJAdHlwZSI6ICJkaWQ6c292OkJ6Q2JzTlloTXJqSGlxWkRUVUFTSGc7c3BlYy9jb25uZWN0aW9ucy8xLjAvaW52aXRhdGlvbiIsICJAaWQiOiAiNjkxYTQ4ZTAtZDBlMi00ODU5LWFlM2ItMGMxYTEzNTkxZGUxIiwgInJlY2lwaWVudEtleXMiOiBbIjZhYVZZU2ZmNG94SktqdUhlVThjVEhEb1l6ZjZKQ1B1S05VWnBMeDd3cnBoIl0sICJsYWJlbCI6ICJBcmllcyBDbG91ZCBBZ2VudCIsICJzZXJ2aWNlRW5kcG9pbnQiOiAiaHR0cDovL2ZlMGQtMTI1LTI1My0xNi0xNjQubmdyb2suaW8ifQ==");
        // invitation.service_endpoint = "http://localhost:8200".to_string();
        let invitation = Invitation::Pairwise(invitation);

        // connect with some default vcx mediator
        // println!("connecting")
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
        let mut conn = Connection::create_with_invite("source_id", &wallet, &agency_client, invitation, true)
            .await
            .unwrap();
        conn.connect(&wallet, &agency_client).await.unwrap();

        println!("{:?}", conn.get_state());

        thread::sleep(Duration::from_millis(5000));

        // find response and accept
        conn.find_message_and_update_state(&wallet, &agency_client)
            .await
            .unwrap();

        println!("{:?}", conn.get_state());

        conn.send_generic_message(&wallet, "hellooooo world, ya ya ya").await.unwrap();

        println!("{:?}", conn.to_string().unwrap());

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