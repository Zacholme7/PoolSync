use alloy::primitives::{Address, Log};
use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::eth::BlockNumberOrTag;
use alloy::rpc::types::eth::Filter;
use alloy::transports::BoxTransport;
use anyhow::Result;

const V2_EVENT_SIG: &str = "PairCreated(address,address,address,uint256)";
const V3_EVENT_SIG: &str = "PoolCreated(address,address,uint24,int24,address)";

enum PoolType {
    V2(UniV2Pool),
    V3(UniV3Pool),
}

// Representation of UniswapV2 pool
struct UniV2Pool {}

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



        //let mut requests = Vec::new();
        for range in block_range {
            //requests.push(tokio::task::spawn(self.fetch_block_range(range.0, range.1, &provider)));
            let filter = Filter::new()
                .select(range.0..range.1)
                .events([V2_EVENT_SIG, V3_EVENT_SIG]);
            let logs = provider.get_logs(&filter).await?;
            println!("{:?}", logs);
        }


        // process the results
        Ok(())
    }

    fn process_logs(&self, logs: Vec<Log>) {
        // process all of the logs
        for log in logs {
            // match the log through something
            let pool_type: PoolType = self.get_pool_type(&log).unwrap();
            let parsed_pool = match pool_type {
                PoolType::V2(_) => self.parse_v2_pool(&log).unwrap(),
                PoolType::V3(_) => self.parse_v3_pool(&log).unwrap(),
            };



        }
    }

    fn get_pool_type(&self, log: &Log) -> Result<PoolType> {
        todo!()
    }

    fn parse_v2_pool(&self, log: &Log) -> Result<PoolType>{
        todo!()
    }

    fn parse_v3_pool(&self, log: &Log) -> Result<PoolType> {
        todo!()
    }
}
async fn fetch_block_range(from: u64, to: u64, provider: &RootProvider<BoxTransport>) {
    let filter = Filter::new()
        .select(from..to)
        .events([V2_EVENT_SIG, V3_EVENT_SIG]);
    let logs = provider.get_logs(&filter).await;
    println!("{:?}", logs);
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
