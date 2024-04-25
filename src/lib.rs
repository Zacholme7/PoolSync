use alloy::dyn_abi::abi::token;
use alloy::primitives::Address;
use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::eth::{BlockNumberOrTag, Log, Filter};
use alloy::sol_types::SolEvent;
use alloy::transports::BoxTransport;
use anyhow::Result;
use alloy::sol;
use IERC20::nameReturn;

sol!(
    #[allow(missing_docs)]
    #[derive(Debug)]
    #[sol(rpc)]
    IERC20,
    "abi/IERC20.json"
);

sol!(
    #[derive(Debug)]
    event PairCreated(
        address indexed token0,
        address indexed token1, 
        address pair,
        uint
    );
);

// use event for these
const V2_EVENT_SIG: &str = "PairCreated(address,address,address,uint256)";
const V3_EVENT_SIG: &str = "PoolCreated(address,address,uint24,int24,address)";

enum PoolType {
    V2(UniV2Pool),
    V3(UniV3Pool),
}

// Representation of UniswapV2 pool
#[derive(Debug)]
struct UniV2Pool {
        token0_name: String,
        token1_name: String,
        token0_address: Address,
        token1_address: Address,
        pair: Address,
}

/// Representation of UniswapV3 pool
struct UniV3Pool {}

#[derive(Debug, PartialEq)]
pub struct Scanner {
    block_from: BlockNumberOrTag,
    block_to: BlockNumberOrTag,
    token_0: Option<Address>,
    token_1: Option<Address>,
    uni_v2: bool,
    uni_v3: bool,
}

impl Scanner {
    pub async fn scan_pools(&self, provider: &RootProvider<BoxTransport>) -> Result<()> {
        // break blocks into 2k ranges
        let mut blocks_processed: u64 = 0;
        let mut block_range = Vec::new();
        let block_from = self.block_from.as_number().unwrap();
        let block_to = self.block_to.as_number().unwrap();
        loop {
            let start_block = block_from + blocks_processed;
            let mut end_block = start_block + 2000 - 1;
            if end_block > block_to {
                end_block = block_to;
                block_range.push((start_block, end_block));
                break;
            }
            block_range.push((start_block, end_block));
            blocks_processed += 2000;
        }


        // fetch all the logs for the block range
        for range in block_range {
            // create filter for the range and events
            let filter = Filter::new()
                .select(range.0..range.1)
                .events([PairCreated::SIGNATURE]); //, V3_EVENT_SIG]);
            // fetch and process the logs
            let logs = provider.get_logs(&filter).await?;
            let pools = self.process_logs(logs, &provider).await?;
            println!("{:?}",  pools);
        }

        // process the results
        Ok(())
    }

    async fn process_logs(&self, logs: Vec<Log>, provider: &RootProvider<BoxTransport>) -> Result<Vec<UniV2Pool>>{
        let mut pools: Vec<UniV2Pool> = Vec::new();
        for log in logs {
            // match the log through something
            let pair: PairCreated = SolEvent::decode_log_data(&log.inner.data, true)?;
            let (token0_name, token1_name) = self.get_token_names(provider, pair.token0, pair.token1).await?;
            let pool = UniV2Pool {
                token0_address: pair.token0,
                token1_address: pair.token1,
                token0_name: token0_name,
                token1_name: token1_name,
                pair: pair.pair
            };
            pools.push(pool);
       }
        Ok(pools)
    }

    async fn get_token_names(&self, provider: &RootProvider<BoxTransport>, token0: Address, token1: Address) -> Result<(String, String)> {
            let token_1 = IERC20::new(token0, &provider);
            let token_2 = IERC20::new(token1, &provider);
            let token1_name  = token_1.name().call().await?;
            let token2_name = token_2.name().call().await?;
            Ok((token1_name._0, token2_name._0))
    }
}









#[derive(Debug)]
pub struct ScannerBuilder {
    block_from: Option<BlockNumberOrTag>,
    block_to: Option<BlockNumberOrTag>,
    token_0: Option<Address>,
    token_1: Option<Address>,
    uni_v2: bool,
    uni_v3: bool,
}

impl ScannerBuilder {
    pub fn new() -> ScannerBuilder {
        Self {
            block_from: Some(BlockNumberOrTag::Number(15_000_000)),
            block_to: Some(BlockNumberOrTag::Latest),
            token_0: None,
            token_1: None,
            uni_v2: false,
            uni_v3: false,
        }
    }

    /// Set the starting block, defaults to block 15 million
    pub fn block_from(mut self, from: BlockNumberOrTag) -> ScannerBuilder {
        self.block_from = Some(from);
        self
    }

    // Set the ending block, defaults to the latest block
    pub fn block_to(mut self, to: BlockNumberOrTag) -> ScannerBuilder {
        self.block_to = Some(to);
        self
    }

    /// Specifiy a specific token for the first token
    pub fn token_0(mut self, token: Address) -> ScannerBuilder {
        self.token_0 = Some(token);
        self
    }

    /// Specifiy a specific token for the second token
    pub fn token_1(mut self, token: Address) -> ScannerBuilder {
        self.token_1 = Some(token);
        self
    }

    /// Enable uniswap v2 pools
    pub fn uni_v2(mut self) -> ScannerBuilder {
        self.uni_v2 = true;
        self
    }

    /// Enable uniswap v3 pools
    pub fn uni_v3(mut self) -> ScannerBuilder {
        self.uni_v3 = true;
        self
    }

    /// Finalize and construct the Pool Scanner
    pub fn finalize(self) -> Scanner {
        Scanner {
            block_from: self.block_from.unwrap(),
            block_to: self.block_to.unwrap(),
            token_0: self.token_0,
            token_1: self.token_1,
            uni_v2: self.uni_v2,
            uni_v3: self.uni_v3
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn builder_test() {
        let scanner_direct = Scanner {
            block_from: BlockNumberOrTag::Number(15_000_000),
            block_to: BlockNumberOrTag::Number(17_000_000),
            token_0: None,
            token_1: None,
            uni_v2: false,
            uni_v3: false,
        };
        let scanner_from_builder = ScannerBuilder::new()
            .block_from(15_000_000.into())
            .block_to(17_000_000.into())
            .finalize();
        assert_eq!(scanner_direct, scanner_from_builder);
    }
}
