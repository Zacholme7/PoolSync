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

use pool_scanner::{Scanner, ScannerBuilder};

#[tokio::main]
async fn main() -> Result<()> {
        // load the dotenv
        dotenv::dotenv().ok();
        
        // setup anvil instance
        let rpc_url = std::env::var("RPC_URL")?;
        let anvil = Anvil::new().fork(rpc_url).try_spawn()?;

        // setup provier
        let anvil_endpoint = anvil.endpoint().parse()?;
        let provider = ProviderBuilder::new().on_http(anvil_endpoint)?;

        // setup the scanner
        let scanner = ScannerBuilder::new()
                .block_from(15_000_000.into())
                .finalize();

        scanner.scan_pools(&provider).await?;

    Ok(())
}
