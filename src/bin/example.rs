//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.

use alloy::network::EthereumWallet;
use alloy::providers::ProviderBuilder;
use alloy_node_bindings::anvil::Anvil;
use anyhow::Result;
use pool_sync::filter::filter_top_volume;
use pool_sync::{Chain, PoolInfo, PoolSync, PoolType};
use std::sync::Arc;

use alloy::primitives::address;
use alloy::signers::local::PrivateKeySigner;
/// The main entry point for the pool synchronization program.
///
/// This function performs the following steps:
/// 1. Loads environment variables
/// 2. Constructs an Alloy provider for the specified chain
/// 3. Configures and builds a PoolSync instance
/// 4. Initiates the pool synchronization process
/// 5. Prints the number of synchronized pools
///
/// # Errors
///
/// This function will return an error if:
/// - The required environment variables are not set
/// - There's an issue constructing the provider or PoolSync instance
/// - The synchronization process fails
use alloy::sol_types::{sol, SolInterface};

sol! {
    #[derive(Debug)]
    #[sol(rpc, bytecode = "608080604052346015576106cb908161001a8239f35b5f80fdfe60806040526004361015610011575f80fd5b5f3560e01c63cc90d2cd14610024575f80fd5b346103745760203660031901126103745760043567ffffffffffffffff8111610374573660238201121561037457806004013561006081610523565b9161006e60405193846104ed565b8183526024602084019260051b8201019036821161037457602401915b8183106104cd5783518461009e82610523565b916100ac60405193846104ed565b8083526100bb601f1991610523565b015f5b8181106104b65750505f5b81518110156103d4576001600160a01b036100e482846105ae565b5116906100ef61055f565b91604051630dfe168160e01b8152602081600481855afa9081156102b7575f916103b6575b506001600160a01b0316835260405163d21220a760e01b815290602082600481845afa9182156102b7575f92610380575b506001600160a01b0390911660208401908152604051630240bc6b60e21b81529091606090829060049082905afa9081156102b7575f905f9261031c575b506001600160701b0391821660e08601521660c084015282516040516395d89b4160e01b81526001600160a01b03909116905f81600481855afa9182156102b7576004926020925f91610302575b5060408701526040519283809263313ce56760e01b82525afa80156102b75760ff915f916102e4575b50166080840152516040516395d89b4160e01b81526001600160a01b0390911692905f81600481875afa9384156102b7576004946020925f916102c2575b50606084015260405163313ce56760e01b815294859182905afa9283156102b75760019360ff915f91610289575b501660a082015261027782866105ae565b5261028281856105ae565b50016100c9565b6102aa915060203d81116102b0575b6102a281836104ed565b81019061067c565b87610266565b503d610298565b6040513d5f823e3d90fd5b6102de91503d805f833e6102d681836104ed565b810190610609565b88610238565b6102fc915060203d81116102b0576102a281836104ed565b876101fa565b61031691503d805f833e6102d681836104ed565b896101d1565b9150506060813d8211610378575b81610337606093836104ed565b8101031261037457610348816105f5565b906040610357602083016105f5565b91015163ffffffff81160361037457906001600160701b03610183565b5f80fd5b3d915061032a565b60049192506103a760609160203d81116103af575b61039f81836104ed565b8101906105d6565b929150610145565b503d610395565b6103ce915060203d81116103af5761039f81836104ed565b86610114565b826040518091602082016020835281518091526040830190602060408260051b8601019301915f905b82821061040c57505050500390f35b919360019193955060208091603f19898203018552875190848060a01b038251168152848060a01b0383830151168382015260e06001600160701b03816104796104676040870151610100604088015261010087019061053b565b6060870151868203606088015261053b565b9460ff608082015116608086015260ff60a08201511660a08601528260c08201511660c086015201511691015296019201920185949391926103fd565b6020906104c161055f565b828287010152016100be565b82356001600160a01b03811681036103745781526020928301920161008b565b90601f8019910116810190811067ffffffffffffffff82111761050f57604052565b634e487b7160e01b5f52604160045260245ffd5b67ffffffffffffffff811161050f5760051b60200190565b805180835260209291819084018484015e5f828201840152601f01601f1916010190565b60405190610100820182811067ffffffffffffffff82111761050f576040525f60e083828152826020820152606060408201526060808201528260808201528260a08201528260c08201520152565b80518210156105c25760209160051b010190565b634e487b7160e01b5f52603260045260245ffd5b9081602091031261037457516001600160a01b03811681036103745790565b51906001600160701b038216820361037457565b6020818303126103745780519067ffffffffffffffff8211610374570181601f820112156103745780519067ffffffffffffffff821161050f576040519261065b601f8401601f1916602001856104ed565b8284526020838301011161037457815f9260208093018386015e8301015290565b90816020910312610374575160ff81168103610374579056fea2646970667358221220db83f2d91a6b4dd4d2d0c796048557d0daf58831aebc4080eab5edc755155e8d64736f6c634300081a0033")]
    UniswapV2DataSync,
    "src/abi/UniswapV2DataSync.json"

}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from a .env file if present
    dotenv::dotenv().ok();

    // Construct an Alloy provider/forked anvil instance for the chain you want to sync from
    let url = std::env::var("ETH")?;

    // anvil config
    let anvil = Anvil::new().fork(url).try_spawn()?;
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = Arc::new(
        ProviderBuilder::new()
            .network::<alloy::network::AnyNetwork>()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(anvil.endpoint_url()),
    );

    // deploy and create contract instance
    let contract = UniswapV2DataSync::deploy(&provider).await?;

    println!("Deployed contract at address: {}", contract.address());

    let mut pool_addresses = Vec::new();
    for i in 1..=500 {
        pool_addresses.push(address!("3fd4Cf9303c4BC9E13772618828712C8EaC7Dd2F"));
    }

    let res = contract.syncPoolData(pool_addresses).call().await?;
    println!("{:?}", res);

    /*
    let UniswapV2DataSync::syncPoolDataReturn {
        token0Address,
        token1Address,
        token0Symbol,
        token1Symbol,
        token0Decimals,
        token1Decimals,
        reserve0,
        reserve1,

    } =
    */

    /*
    let provider = Arc::new(
        ProviderBuilder::new()
            .network::<alloy::network::AnyNetwork>()
            .with_recommended_fillers()
            .on_http(anvil.endpoint_url()),
    );

    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2) // Add all the pools you would like to sync
        .chain(Chain::Ethereum) // Specify the chain
        .rate_limit(20) // Specify the rate limit
        .build()?;

    // Initiate the sync process
    let pools = pool_sync.sync_pools(provider.clone()).await?;
    println!("Number of synchronized pools: {}", pools.len());

    // Can get common info
    //for pool in &pools {
    //println!("Pool Address {:?}, Token 0: {:?}, Token 1: {:?}", pool.address(), pool.token0(), pool.token1());
    //}
    */

    // extract all pools with top volume tokens
    //let pools_over_top_volume_tokens = filter_top_volume(pools, 10).await?;

    Ok(())
}
