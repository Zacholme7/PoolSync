use anyhow::Result;
use tracing::info;

mod pool_type;
mod pooldexer;

use pooldexer::Pooldexer;
#[tokio::main]
async fn main() -> Result<()> {
    info!("Starting pooldexer...");

    // read from a config where we have all of the pools we want to sync
    // has the endpoints
    // and has the common database file

    //let pooldexer = Pooldexer::new();
    //pooldexer.index_pools().await?;

    Ok(())
}
