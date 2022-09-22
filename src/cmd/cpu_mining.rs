use std::{future, net::SocketAddr};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
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
use crate::utils::sender::Wrapper;

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
        //let threads = self.threads.unwrap_or(num_cpus::get() as u16);
        //self.run_client().await?;
        let mut wrapper = Wrapper::new();

        let client = Client::new(pool_server, address);
        if let Err(error) = client.start(wrapper).await {
            error!("[Stratum client start] error=> {}", error);
        }

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