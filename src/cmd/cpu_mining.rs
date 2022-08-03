use std::{future, net::SocketAddr};
use log::{info, error};
use structopt::StructOpt;
use anyhow::Result;
use anyhow::{anyhow, bail};
use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::prelude::Address;
use tokio::task;

use crate::stratum::client;
use crate::stratum::client::Client;

#[derive(Debug, StructOpt)]
/// CPU mining command will use cups to mine the proof.
pub struct Cmd {
    #[structopt(short, long)]
    /// The miner address (aleo1...)
    address: Address<Testnet2>,

    #[structopt(short="s", long="pool-server")]
    /// Ip:port of the pool server
    pool_server: SocketAddr,

    /// Worker is a thread pool used to calculate proof
    #[structopt(short="w", long="worker", default_value = "1")]
    worker: u8,

    /// Number of threads that every worker will use
    /// It is recommended to ensure
    /// `worker * thread-per-worker` < `amount of threads of your device`
    #[structopt(short="t", long="threads", default_value = "4")]
    #[structopt(verbatim_doc_comment)]
    threads: u8,
}

impl Cmd {
    pub async fn run(&self) -> Result<()> {
        self.run_client().await?;

        std::future::pending::<()>().await;
        Ok(())
    }

    async fn run_client(&self)->Result<()> {
        let pool_server = self.pool_server.clone();
        let address = self.address;
        task::spawn( async move {
            let client = Client::new();
            if let Err(error) = client.start(&pool_server, &address).await {
                error!("[client start] error {}", error);
            }
        });

        Ok(())
    }
}