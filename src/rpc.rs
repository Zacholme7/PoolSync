


pub struct Rpc;

impl Rpc {
        pub fn fetch_pool_addrs(start_block: u64, end_block: u64, provder: Blah) -> Vec<Address> {
                // this is the same for all pools
                // just have to use the fethcer to get addrress and stuff and can decode the address
                //this 





                todo!()
        }


        pub fn populate_pools(pools_addrs: Vec<Address> ) -> Vec<Pool> {
                // I need to break it all up a,nd then some



        }
}       /* 
                let handles: Vec<_> = (start_block..=end_block)
                    .step_by(step_size as usize)
                    .map(|from_block| {
                        let to_block = (from_block + step_size - 1).min(end_block);
                        self.fetch_and_process_block_range(
                            provider.clone(),
                            rate_limiter.clone(),
                            self.chain,
                            from_block,
                            to_block,
                            fetcher.clone(),
                            progress_bar.clone(),
                        )
                    })
                    .collect();
                let pools_with_addr = join_all(handles).await;
                pools.extend(pools_with_addr.into_iter().flatten());
                */


                /*
                
                  async fn fetch_and_process_block_range<P, T, N>(
        &self,
        provider: Arc<P>,
        semaphore: Arc<Semaphore>,
        chain: Chain,
        from_block: u64,
        to_block: u64,
        fetcher: Arc<dyn PoolFetcher>,
        progress_bar: ProgressBar,
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let _permit = semaphore.acquire().await.unwrap();

        let filter = Filter::new()
            .address(fetcher.factory_address(chain))
            .event(fetcher.pair_created_signature())
            .from_block(from_block)
            .to_block(to_block);

        let logs = provider.get_logs(&filter).await.unwrap();
        let pools = join_all(
            logs.iter()
                .map(|log| async { fetcher.from_log(&log.inner).await }),
        )
        .await
        .into_iter()
        .flatten()
        .collect();

        progress_bar.inc(1);
        pools
    }

    pub async fn populate_pool_data_helper<P, T, N>(
        provider: Arc<P>,
        cache: Vec<Address>,
        semaphore: Arc<Semaphore>,
        fetcher: Arc<dyn PoolFetcher>,
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let _permit = semaphore.acquire().await.unwrap();

        let deployer = UniswapV2DataSync::deploy_builder(provider, cache);
        let res = deployer.call().await.unwrap();
        let constructor_return = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::String,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::String,
            DynSolType::Uint(112),
            DynSolType::Uint(112),
        ])));
        let return_data_tokens = constructor_return.abi_decode_sequence(&res).unwrap();

        let mut pools = Vec::new();
        if let Some(tokens_arr) = return_data_tokens.as_array() {
            for token in tokens_arr {
                if let Some(tokens) = token.as_tuple() {
                    let pool = fetcher.construct_pool_from_data(tokens);
                    pools.push(pool);
                }
            }
        }
        pools
    }

    pub async fn populate_pool_data<P, T, N>(&self, provider: Arc<P>, cache: &mut PoolCache)
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        // collect the pool addresses and separate them into chuncks
        let pool_addresses: Vec<Address> = cache.pools.iter().map(|p| p.address()).collect();
        
        let addr_chunks: Vec<Vec<Address>> = pool_addresses
            .chunks(5)
            .map(|chunk| chunk.to_vec())
            .collect();




        let mut handles = Vec::new();

        let rate_limiter = Arc::new(Semaphore::new(self.rate_limit));

        let fetcher = self.fetchers[&cache.pool_type].clone();
        for chunk in addr_chunks {
            let provider_clone = provider.clone();
            let handle = tokio::task::spawn(PoolSync::populate_pool_data_helper(
                provider.clone(),
                chunk,
                rate_limiter.clone(),
                fetcher.clone(),
            ));
            handles.push(handle);
        }

        let mut data_pools = Vec::new();
        let results = join_all(handles).await;
        for res in results {
            if let Ok(res) = res {
                data_pools.extend(res);
            }
        }
        println!("data_pools: {}", data_pools.len());
    }
                
                 */