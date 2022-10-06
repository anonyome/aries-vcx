#[cfg(test)]
#[cfg(feature = "temp_gm_tests")]
mod integration_tests {
    use aries_vcx::libindy::utils::pool::PoolConfig;
    use aries_vcx::messages::connection::did::Did;
    use aries_vcx::plugins::anoncreds;
    use aries_vcx::plugins::anoncreds::base_anoncreds::BaseAnonCreds;
    use aries_vcx::plugins::anoncreds::credx_anoncreds::IndyCredxAnonCreds;
    use aries_vcx::plugins::ledger::base_ledger::BaseLedger;
    use aries_vcx::plugins::ledger::indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerPool};
    use aries_vcx::plugins::wallet::agency_client_wallet::ToBaseAgencyClientWallet;
    use indy_vdr::config::PoolConfig as IndyVdrPoolConfig;
    use indy_vdr::pool::{PoolBuilder, PoolTransactions};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use agency_client::agency_client::AgencyClient;
    use aries_vcx::{
        core::profile::{indy_profile::IndySdkProfile, profile::Profile},
        global::{self, settings},
        handlers::connection::connection::Connection,
        libindy::utils::wallet::WalletConfig,
        utils::devsetup::{AGENCY_DID, AGENCY_VERKEY},
    };
    use indy_sys::WalletHandle;

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
    async fn test_ledger_fetch() {
        // ----------- try with indyvdr
        let (_, indy_handle, _, _, _) = setup_with_existing_conn().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile); // just for the wallet

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

        println!(
            "service; {:?}",
            ledger.get_service(&Did::new(pub_did).unwrap()).await.unwrap()
        );

        ()
    }

    #[tokio::test]
    async fn compare_vdr_to_sdk_fns_temp() {
        let (_, indy_handle, _, _, _) = setup_with_existing_conn().await;
        let indy_profile = IndySdkProfile::new(indy_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile); // just for the wallet

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
        let vdr_ledger = IndyVdrLedger::new(profile.clone(), indy_vdr_pool);
        let credx_anoncreds = IndyCredxAnonCreds::new(Arc::clone(&profile));

        let indy_sdk_ledger = profile.clone().inject_ledger();
        let indy_sdk_anoncreds = profile.clone().inject_anoncreds();

        let rev_id = "D6EMVkDnBmuMCtZGwjgR9A:4:D6EMVkDnBmuMCtZGwjgR9A:3:CL:88813:Dummy_Uni_Transaction:CL_ACCUM:ec86da86-b4ce-45f6-afeb-d0c2e71e36b3";

        let cred_def_id = "D6EMVkDnBmuMCtZGwjgR9A:3:CL:88813:Dummy_Uni_Transaction";

        // println!("vdr; {}\n", vdr_ledger.get_rev_reg_def_json(rev_id).await.unwrap());
        // println!("indy; {}", indy_sdk_ledger.get_rev_reg_def_json(rev_id).await.unwrap());

        // println!("vdr; {:?}\n", vdr_ledger.get_rev_reg_delta_json(rev_id, None, None).await.unwrap());
        // println!("indy; {:?}", indy_sdk_ledger.get_rev_reg_delta_json(rev_id, None, None).await.unwrap());

        println!(
            "vdr; {}\n",
            credx_anoncreds.get_cred_def(None, cred_def_id).await.unwrap().1
        );
        println!(
            "indy; {}",
            indy_sdk_anoncreds.get_cred_def(None, cred_def_id).await.unwrap().1
        );
    }

    #[tokio::test]
    async fn test_anoncreds_rand_functionality() {
        let (_, indy_handle, indy_profile, profile, _) = setup_with_existing_conn().await;

        let indysdk_anoncreds = Arc::clone(&profile).inject_anoncreds();
        let credx_anoncreds = IndyCredxAnonCreds::new(Arc::clone(&profile));

        let ms = credx_anoncreds.prover_create_master_secret("abc").await.unwrap();

        println!("{:?}", ms);

        let ms = indysdk_anoncreds.prover_create_master_secret("abc").await.unwrap();

        println!("{:?}", ms);

        ()
    }

    mod helper {}
}
