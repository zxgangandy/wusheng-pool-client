use std::{future, net::SocketAddr};
use log::{info};
use structopt::StructOpt;
use anyhow::Result;
use snarkvm::console::account::address::Address;
use snarkvm::prelude::Testnet3;
//use snarkvm::dpc::testnet2::Testnet2;
//use snarkvm::prelude::Address;

#[derive(Debug, StructOpt)]
/// CPU mining command will use cups to mine the proof.
pub struct Cmd {
    #[structopt(short, long)]
    /// The miner address (aleo1...)
    address: Address<Testnet3>,

    #[structopt(short="s", long="pool-server")]
    /// Ip:port of the pool server
    pool_server: SocketAddr,

    /// Worker is a thread pool used to calculate proof
    #[structopt(short="w", long="worker", default_value = "1")]
    worker: u8,

    /// Number of threads that every worker will use
    /// It is recommended to ensure
    /// `worker * thread-per-worker` < `amount of threads of your device`
    #[structopt(short="threads", long="thread-per-worker", default_value = "4")]
    #[structopt(verbatim_doc_comment)]
    threads: u8,
}

impl Cmd {
    pub async fn run(&self) -> Result<()> {


        std::future::pending::<()>().await;
        Ok(())
    }
}