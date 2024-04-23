use anyhow::Result;
use alloy::{
    network::EthereumSigner, node_bindings::Anvil, primitives::U256, providers::ProviderBuilder,
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
        let anvil_endpoint = anvil.endpoint();
        let provider = ProviderBuilder::new().on_builtin(&anvil_endpoint).await?;

        // setup the scanner
        let scanner = ScannerBuilder::new()
                .block_from(15_000_000.into())
                .block_to(16_000_000.into())
                .uni_v2()
                .finalize();

        scanner.scan_pools(&provider).await?;
        Ok(())
}
