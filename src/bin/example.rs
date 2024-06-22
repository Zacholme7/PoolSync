use anyhow::Result;
use alloy::providers::ProviderBuilder;
use pool_sync::{PoolSync, PoolType};
use alloy::primitives::{U256,U128};
use alloy::node_bindings::Anvil;
use alloy::sol_types::{sol, SolCall};
use foundry_evm::fork::{BlockchainDb, BlockchainDbMeta, SharedBackend};
use revm::interpreter::primitives::{keccak256, AccountInfo, Bytecode, Bytes, TransactTo};
use revm::{db::CacheDB, Evm};
use revm_primitives::{Address, ExecutionResult, Output};
use alloy::providers::Provider;
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;
use std::collections::BTreeSet;
use std::sync::Arc;

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV2Sync,
    "contracts/out/UniswapV2Sync.sol/UniswapV2Sync.json"
);

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://eth.merkle.io";
    let url = "https://eth-mainnet.public.blastapi.io";
    let anvil = Anvil::new().fork(url).try_spawn()?;
    let wallet = EthereumWallet::from(signer);
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    // Create a provider with the wallet.
    let rpc_url = anvil.endpoint().parse()?;
    let provider =
        Arc::new(ProviderBuilder::new()
        .with_recommended_fillers()
        .network::<alloy::network::AnyNetwork>()
        .wallet(wallet).on_http(rpc_url));


    let contract = UniswapV2Sync::deploy(&provider).await?;
    let block_number = provider.get_block_number().await?;


    // setup shared backend
    let shared_backend = SharedBackend::spawn_backend_thread(
        provider.clone(),
        BlockchainDb::new(
            BlockchainDbMeta {
                cfg_env: Default::default(),
                block_env: Default::default(),
                hosts: BTreeSet::from(["".to_string()]),
            },
            None,
        ),
        Some(block_number.into()),
    );

    let db = CacheDB::new(shared_backend);
    let mut evm = Evm::builder().with_db(db).build();

    // modify the env
    evm.cfg_mut().limit_contract_code_size = Some(0x100000);
    evm.cfg_mut().disable_block_gas_limit = true;
    evm.cfg_mut().disable_base_fee = true;
    evm.block_mut().number = U256::from(block_number + 1);

    // Simulate getAllPairs call
    let start = U256::from(0);
    let end = U256::from(5000);
    let calldata = contract.getAllPairs(start, end).encode_input().unwrap();

    let result = evm.transact_ref(TransactTo::Call(contract_address.into()), calldata, None)?;

    if let ExecutionResult::Success { output, .. } = result {
        // Decode the output (implement proper decoding based on your contract's output format)
        let pairs: Vec<Address> = output.chunks(32).map(|chunk| Address::from_slice(&chunk[12..])).collect();
        println!("First 10 pair addresses: {:?}", &pairs[..10.min(pairs.len())]);
        println!("Total pairs fetched: {}", pairs.len());
    } else {
        println!("EVM execution failed");
    }



    // build a PoolSync and then sync pools
    /* 
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2)
        .build();
    pool_sync.sync_pools(&provider).await;
    */

    Ok(())
}
