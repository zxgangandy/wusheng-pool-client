use std::{future, net::SocketAddr};
use std::str::FromStr;
use log::{info, error};
use structopt::StructOpt;
use anyhow::Result;
use anyhow::{anyhow, bail};
//use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::prelude::Address;
//use snarkvm::traits::Network;
use tokio::task;

use crate::stratum::client;
use crate::stratum::client::Client;

#[derive(Debug, StructOpt)]
/// CPU mining command will use cups of devices to mine the proof.
pub struct Cmd {
    #[structopt(short, long)]
    /// The miner address (aleo1...)
    address: String,

    #[structopt(short="s", long="pool-server")]
    /// Ip:port of the pool server
    pool_server: SocketAddr,

    // /// Worker is a thread pool used to calculate proof
    // #[structopt(short="w", long="worker", default_value = "1")]
    // worker: u8,

    /// Number of threads that every miner will use
    /// It is recommended to ensure
    /// `miner * thread-per-miner` < `amount of threads of your device`
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
        let address = self.address.clone();
        task::spawn( async move {
            let client = Client::new(pool_server, address);
            if let Err(error) = client.start().await {
                error!("[Stratum client start] error=> {}", error);
            }
        });

        Ok(())
    }

    // fn get_address<N>(&self, address: &String) -> Address<N> where N: Network {
    //     let address = Address::<N>::from_str(address).unwrap();
    //     return address;
    // }
}