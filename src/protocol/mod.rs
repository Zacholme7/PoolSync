// declare our modules
mod constants;
pub mod traits;
pub mod uniswap_v2;

// import everyting necessary for macro generation
use crate::errors::*;
use crate::protocol::traits::{AutomatedMarketMaker, AutomatedMarketMakerFactory};
use crate::protocol::uniswap_v2::{UniswapV2Factory, UniswapV2Pool};
use crate::{amm, factory};
use alloy_network::Network;
use alloy_provider::Provider;
use alloy_transport::Transport;
use alloy_primitives::Address;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// generate the amms
amm!(UniswapV2Pool); // add other amms

// generate the factories
factory!(UniswapV2Factory); // add other factories
