use crate::pools::PoolType;
use alloy::primitives::{address, Address};
use std::collections::HashMap;

/// Holds mappings from a PoolType to its factory address
struct PoolFactory {
    factories: HashMap<PoolType, Address>,
}

impl PoolFactory {
    // Construct a new poolfactory that contains all the factory addresses for protocols supported
    fn new() -> Self {
        let mut factories = HashMap::new();

        // insert all of the factories
        factories.insert(
            PoolType::UniswapV2,
            address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
        );
        //factories.insert(PoolType::UniswapV3, address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"));
        //...

        Self { factories }
    }
}
