use anyhow::Result;

use alloy::providers::{Provider, RootProvider};
use alloy::pubsub::PubSubFrontend;
use alloy::rpc::client::WsConnect;
use alloy::rpc::types::eth::Filter;
use alloy::rpc::types::eth::BlockNumberOrTag;
use alloy::{
    network::EthereumSigner, node_bindings::Anvil, primitives::U256, providers::ProviderBuilder,
    signers::wallet::LocalWallet, sol,
};

#[tokio::main]
async fn main() -> Result<()> {
/* 
    // spin up forked anvil node
    dotenv::dotenv().ok();
    let rpc_url = std::env::var("RPC_URL")?;
    let anvil = Anvil::new().fork(rpc_url).try_spawn()?;

    // setup signer from first local account
    let signer: LocalWallet = anvil.keys()[0].clone().into();

    // create an http provider
    let rpc_url = anvil.endpoint().parse()?;
    let http_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(signer))
        .on_http(rpc_url)?;

    // make out filter
    let pair_created_event = "PairCreated(address,address,address,uint256)";
    let swap_event = "Swap(address,address,int256,int256,uint160,uint128,int24)";
    let pool_created = "createPool(address,address,uint24)";
    let v3 = "PoolCreated(address,address,uint24,int24,address)";
    let filter = Filter::new().select(19652100..19682560).event(v3);
    let logs = http_provider.get_logs(&filter).await?;
    println!("{:?}", logs);
    */

    Ok(())
}
