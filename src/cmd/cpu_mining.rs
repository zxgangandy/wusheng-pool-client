use std::{future, net::SocketAddr};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use log::{info, error};
use structopt::StructOpt;
use anyhow::Result;
use anyhow::{anyhow, bail};
use snarkvm::prelude::Address;
use tokio::task;
use crate::mining::mgr::Manager;

use crate::stratum::client;
use crate::stratum::client::Client;
use crate::utils::global;

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
    #[structopt(short="n", long="num_miner", default_value = "1")]
    num_miner: u8,

    // /// Number of threads that every thread pool will use
    // /// It is recommended to ensure
    // /// `max_core * threads` < `amount of threads of your device`
    // #[structopt(short="n", long="threads")]
    // #[structopt(verbatim_doc_comment)]
    // threads: Option<u16>,
}

impl Cmd {
    pub async fn run(&self) -> Result<()> {
        info!("Start cpu mining command!!!");
        let mut senders = global::Senders::new();

        let client = Client::new(self.pool_server, self.address.clone());
        if let Err(error) = client.start(senders.clone()) {
            error!("[Stratum client start] error=> {}", error);
        }

        info!("Stratum client started!!!");

        let mut mgr = Manager::new(senders);
        if let Err(error) =  mgr.start_cpu(
            self.num_miner,
            self.address.clone(),
            self.pool_server).await
        {
            error!("[Mining manager start] error=> {}", error);
        }

        info!("Mining manager started!!!");
        Ok(())
    }
}