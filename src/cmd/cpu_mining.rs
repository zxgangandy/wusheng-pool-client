use std::{future, net::SocketAddr};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use log::{info, error};
use structopt::StructOpt;
use anyhow::Result;
use anyhow::{anyhow, bail};
use snarkvm::prelude::Address;
use tokio::task;
use crate::mining::prover::Prover;

use crate::stratum::client;
use crate::stratum::client::Client;
use crate::storage::Storage;

#[derive(Debug, StructOpt)]
/// CPU mining command will use cups of devices to mine the proof.
pub struct Cmd {
    #[structopt(short, long)]
    /// The miner address (aleo1...)
    address: String,

    #[structopt(short="server", long="pool-server")]
    /// Ip:port of the pool server
    pool_server: SocketAddr,

    /// max_core is the count of the thread pool used to calculate proof
    #[structopt(short="n", long="num_worker", default_value = "1")]
    num_worker: u8,
}

impl Cmd {
    pub async fn run(&self) -> Result<()> {
        info!("Start cpu mining command!!!");
        let mut senders = Storage::new();

        let client = Client::new(self.pool_server, self.address.clone());
        if let Err(error) = client.start(senders.clone()) {
            error!("[Stratum client start] error=> {}", error);
        }

        info!("Stratum client started!!!");

        let mut prover = Prover::new(senders, self.address.clone()).await;
        if let Err(error) =  prover.start_cpu(
            self.num_worker,
            self.address.clone(),
            self.pool_server).await
        {
            error!("[Prover start] error=> {}", error);
        }

        info!("Prover started!!!");
        Ok(())
    }
}