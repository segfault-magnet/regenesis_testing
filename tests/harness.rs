#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    use fuels::fuel_node::Config;
    use fuels::prelude::setup_program_test;
    use fuels::test_helpers::DbType;
    use fuels::{
        prelude::{WalletUnlocked, BASE_ASSET_ID},
        test_helpers::{setup_single_asset_coins, setup_test_provider},
    };

    use fuels::prelude::*;

    #[test]
    fn test_name() {
        // STEP 1 INITIAL WORKINGS OF THE CHAIN
        let wallet = WalletUnlocked::new_from_mnemonic_phrase(
            "humble twelve glad tree gold left limit among pioneer dress assist inch",
            None,
        )
        .unwrap();

        let db_path = PathBuf::from("./fuel_db");
        let snapshot_file = PathBuf::from("./snapshot.json");
        std::fs::remove_dir_all(&db_path).ok();
        std::fs::remove_file(&snapshot_file).ok();

        assert_eq!(
            wallet.address().hash().to_string(),
            "5349930da01b2b94e137c035a499ceff4314e0259f0552ec9e2b5156d8007958"
        );

        setup_program_test!(
            Abigen(Contract(name = "MyContract", project = "./some_contract")),
        );

        let contract = Contract::load_from("./some_contract/out/debug/some_contract.bin", Default::default()).unwrap();
        let contract_id = contract.contract_id();

        let amount = 100_000;
        let coins = 10;
        let counter = 10;

        tokio::runtime::Runtime::new()
            .expect("Tokio runtime failed")
            .block_on(async {
                let mut wallet = wallet.clone();
                let coins = setup_single_asset_coins(wallet.address(), BASE_ASSET_ID, coins, amount);
                let config = fuels::test_helpers::Config {
                    database_type: DbType::RocksDb,
                    database_path: db_path.clone(),
                    ..Config::local_node()
                };
                let (provider, _) = setup_test_provider(coins, vec![], Some(config), None).await;

                wallet.set_provider(provider);

                contract.deploy(&wallet, Default::default()).await.unwrap();
                let contract_instance = MyContract::new(contract_id, wallet);

                let methods = contract_instance.methods();
                methods.initialize_counter(counter).call().await.unwrap();

                let stored_value = methods.get_counter().call().await.unwrap().value;
                assert_eq!(stored_value, 10);
            });

        // STEP 2 snapshot the current state
        let file = File::create(snapshot_file.clone()).unwrap();
        let command_output = Stdio::from(file);
        Command::new("./fuel-core/target/debug/fuel-core")
            .args([
                "snapshot",
                "--db-path",
                db_path.to_str().unwrap(),
                "everything",
                "--chain",
                "local_testnet",
            ])
            .stdout(command_output)
            .stderr(Stdio::null())
            .status()
            .unwrap();

        // STEP 3 regenesis
        std::fs::remove_dir_all(&db_path).ok();

        let mut child = Command::new("./fuel-core/target/debug/fuel-core")
            .args([
                "run",
                "--db-path",
                db_path.to_str().unwrap(),
                "--chain",
                snapshot_file.to_str().unwrap(),
                "--port",
                "8081"
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

            std::thread::sleep(Duration::from_secs(2));

            tokio::runtime::Runtime::new()
            .expect("Tokio runtime failed")
            .block_on(async {
                let mut wallet = wallet.clone();
                let provider = Provider::connect("127.0.0.1:8081").await.unwrap();
                wallet.set_provider(provider);

                assert_eq!(wallet.get_asset_balance(&BASE_ASSET_ID).await.unwrap(), amount * coins);

                let contract_instance = MyContract::new(contract_id, wallet);
                let methods = contract_instance.methods();
                let regenesis_counter = methods.get_counter().call().await.unwrap().value;

                assert_eq!(counter, regenesis_counter);
            }
        );

        child.kill().unwrap();
    }
}
