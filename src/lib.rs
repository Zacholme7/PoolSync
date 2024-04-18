use alloy::primitives::Address;
use alloy::rpc::types::eth::BlockNumberOrTag;
use alloy::signers::wallet::yubihsm::Client;
use alloy::transports::http::Http;
use std::convert::From;
use alloy::providers::{Provider, RootProvider};
use alloy::pubsub::PubSubFrontend;
use alloy::rpc::types::eth::Filter;
use anyhow::Result;
use alloy::{
        network::EthereumSigner, node_bindings::Anvil, primitives::U256, providers::ProviderBuilder,
        signers::wallet::LocalWallet, sol,
    };

const V2_EVENT_SIG: &str = "PairCreated(address,address,address,uint256)";
const V3_EVENT_SIG: &str = "PoolCreated(address,address,uint24,int24,address)";
                                
enum PoolType {
        V2(UniV2Pool),
        V3(UniV3Pool),
}

// Representation of UniswapV2 pool
struct UniV2Pool{

}

/// Representation of UniswapV3 pool
struct UniV3Pool{

}

#[derive(Debug, PartialEq)]
pub  struct Scanner {
        block_from: BlockNumberOrTag,
        block_to: BlockNumberOrTag,
        token_0: Option<Address>,
        token_1: Option<Address>
}

impl Scanner {
        pub async fn scan_pools(&self, provider: &RootProvider<Http<Client>>) -> Result<()> {

                let  filter = Filter::new()
                        .select(self.block_from..self.block_to)
                        .events([V2_EVENT_SIG, V3_EVENT_SIG]);
                let logs = provider.get_logs(&filter).await?;
                println!("{:?}", logs);
                Ok(())
        }
}

#[derive(Debug)]
pub struct ScannerBuilder {
        block_from: Option<BlockNumberOrTag>,
        block_to: Option<BlockNumberOrTag>,
        token_0: Option<Address>,
        token_1: Option<Address>
}

impl ScannerBuilder {
        pub fn new() -> ScannerBuilder {
                Self {
                        block_from: Some(BlockNumberOrTag::Number(10_000_000)),
                        block_to: Some(BlockNumberOrTag::Latest),
                        token_0: None,
                        token_1: None,
                }
        }

        pub fn block_from(mut self, from: BlockNumberOrTag) -> ScannerBuilder {
                self.block_from = Some(from);
                self
        }

        pub fn block_to(mut self, to: BlockNumberOrTag) -> ScannerBuilder {
                self.block_to = Some(to);
                self
        }

        pub fn token_0(mut self, token: Address) -> ScannerBuilder {
                self.token_0 = Some(token);
                self
        }

        pub fn token_1(mut self, token: Address) -> ScannerBuilder {
                self.token_1 = Some(token);
                self
        }

        pub fn finalize(self) -> Scanner {
                Scanner {
                        block_from: self.block_from.unwrap(),
                        block_to: self.block_to.unwrap(),
                        token_0: self.token_0,
                        token_1: self.token_1
                }
        }
}


#[cfg(test)]
mod test {
        use super::*;

        #[test]
        fn builder_test() {
                let scanner_direct  = Scanner {
                        block_from: BlockNumberOrTag::Number(15_000_000),
                        block_to: BlockNumberOrTag::Number(17_000_000),
                        token_0: None,
                        token_1: None,
                };
                let scanner_from_builder = ScannerBuilder::new()
                        .block_from(15_000_000.into())
                        .block_to(17_000_000.into())
                        .finalize();
                assert_eq!(scanner_direct, scanner_from_builder);
        }
}






