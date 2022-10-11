#[cfg(test)]
#[cfg(feature = "temp_gm_tests")]
mod integration_tests {
    use aries_vcx::core::profile::modular_wallet_profile::{LedgerPoolConfig, ModularWalletProfile};
    use aries_vcx::libindy::utils::pool::PoolConfig;
    use aries_vcx::messages::connection::did::Did;
    use aries_vcx::plugins::anoncreds;
    use aries_vcx::plugins::anoncreds::base_anoncreds::BaseAnonCreds;
    use aries_vcx::plugins::anoncreds::credx_anoncreds::IndyCredxAnonCreds;
    use aries_vcx::plugins::ledger::base_ledger::BaseLedger;
    use aries_vcx::plugins::ledger::indy_ledger;
    use aries_vcx::plugins::ledger::indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerPool};
    use aries_vcx::plugins::wallet::agency_client_wallet::ToBaseAgencyClientWallet;
    use aries_vcx::plugins::wallet::base_wallet::AsyncFnIteratorCollect;
    use indy_vdr::config::PoolConfig as IndyVdrPoolConfig;
    use indy_vdr::pool::{PoolBuilder, PoolTransactions};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

    async fn indy_global_setup() {
        global::pool::open_main_pool(&PoolConfig {
            genesis_path: settings::DEFAULT_GENESIS_PATH.to_string(),
            pool_name: None,
            pool_config: None,
        })
        .await
        .unwrap();
    }

    async fn setup_profiles() -> (WalletHandle, IndySdkProfile, Arc<dyn Profile>, Arc<dyn Profile>) {
        indy_global_setup().await;
        let indy_handle = open_default_indy_handle().await;
        let _indy_profile = IndySdkProfile::new(indy_handle);
        let indy_wallet = _indy_profile.inject_wallet();
        let indy_profile = Arc::new(_indy_profile);

        let ledger_pool_config = LedgerPoolConfig {
            genesis_file_path:
                "/Users/gmulhearne/Documents/dev/platform/di-edge-agent/edge-agent-core/aries-vcx/aries_vcx/genesis.txn"
                    .to_string(),
        };
        let _mod_profile = ModularWalletProfile::new(indy_wallet, ledger_pool_config).unwrap();
        let mod_profile = Arc::new(_mod_profile);

        return (indy_handle, _indy_profile, indy_profile, mod_profile);
    }

    #[tokio::test]
    async fn test_ledger_fetch() {
        let (indy_handle, _, indy_profile, mod_profile) = setup_profiles().await;
        let vdr_ledger = mod_profile.inject_ledger();
        let indy_ledger = indy_profile.inject_ledger();

        // ----------- try with indyvdr

        println!(
            "VDR NYM: {}\n\n\n",
            vdr_ledger.get_nym("D6EMVkDnBmuMCtZGwjgR9A").await.unwrap()
        );

        println!(
            "VDR CRED DEF: {}\n\n\n",
            vdr_ledger
                .get_cred_def("D6EMVkDnBmuMCtZGwjgR9A:3:CL:88813:Dummy_Uni_Transaction")
                .await
                .unwrap()
        );

        // ----------- try with libindy directly

        println!(
            "DIRECT INDY NYM: {}\n\n\n",
            aries_vcx::libindy::utils::ledger::get_nym("D6EMVkDnBmuMCtZGwjgR9A")
                .await
                .unwrap()
        );

        println!(
            "DIRECT INDY CRED DEF: {}\n\n\n",
            aries_vcx::libindy::utils::ledger::libindy_get_cred_def(
                indy_handle,
                "D6EMVkDnBmuMCtZGwjgR9A:3:CL:88813:Dummy_Uni_Transaction"
            )
            .await
            .unwrap()
        );

        // try with indy indysdkledger

        println!(
            "INDYLEDGER NYM: {}\n\n\n",
            indy_ledger.get_nym("D6EMVkDnBmuMCtZGwjgR9A").await.unwrap()
        );

        println!(
            "INDYLEDGER CRED DEF: {}\n\n\n",
            indy_ledger
                .get_cred_def("D6EMVkDnBmuMCtZGwjgR9A:3:CL:88813:Dummy_Uni_Transaction")
                .await
                .unwrap()
        );

        // other
        let pub_did = "D6EMVkDnBmuMCtZGwjgR9A";

        println!(
            "service; {:?}",
            vdr_ledger.get_service(&Did::new(pub_did).unwrap()).await.unwrap()
        );

        ()
    }

    #[tokio::test]
    async fn compare_vdr_to_sdk_fns_temp() {
        let (indy_handle, _, indy_profile, mod_profile) = setup_profiles().await;

        let vdr_ledger = mod_profile.clone().inject_ledger();
        let credx_anoncreds = mod_profile.clone().inject_anoncreds();

        let indy_sdk_ledger = indy_profile.clone().inject_ledger();
        let indy_sdk_anoncreds = indy_profile.clone().inject_anoncreds();

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
        let (_, _, indy_profile, mod_profile) = setup_profiles().await;

        let credx_anoncreds = IndyCredxAnonCreds::new(Arc::clone(&mod_profile));
        let indysdk_anoncreds = indy_profile.clone().inject_anoncreds();

        let ms = credx_anoncreds.prover_create_master_secret("rand1").await.unwrap();

        println!("{:?}", ms);

        let ms = indysdk_anoncreds.prover_create_master_secret("rand1").await.unwrap();

        println!("{:?}", ms);

        ()
    }

    #[tokio::test]
    async fn test_wallet_search() {
        let (_, _, indy_profile, _) = setup_profiles().await;

        let wallet = indy_profile.inject_wallet();

        let mut iterator1 = wallet.iterate_wallet_records("AAAAAA", "{}", "{}").await.unwrap();

        let x = iterator1.collect().await.unwrap();

        println!("{:?}", x);
        
        let mut iterator2 = wallet
            .iterate_wallet_records("VCX_CREDENTIAL", "{}", "{}")
            .await
            .unwrap();


        let y = iterator2.collect().await.unwrap();

        println!("{:?}", y);

        ()
    }

    mod helper {}
}