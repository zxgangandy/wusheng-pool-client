use std::net::SocketAddr;
use std::time::Duration;

use json_rpc_types::Id;
use tokio::{
    net::TcpStream,
    task,
    time::{sleep, timeout},
};
use tokio_util::codec::Framed;
use futures_util::sink::SinkExt;
use tokio_stream::StreamExt;
use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::prelude::Address;
use log::{error, info};
use anyhow::Result;
use anyhow::{anyhow, bail};
use crate::stratum::codec::StratumCodec;
use crate::stratum::handler::Handler;
use crate::stratum::protocol::StratumProtocol;

pub struct Client{
    pool_server: SocketAddr,
    miner_address: String,
}

impl Client {
    pub fn new(pool_server: SocketAddr, miner_address: String) -> Self {
        Client { pool_server, miner_address}
    }

    /// Start the stratum client
    ///
    /// Preconditions: Client connected the pool server, then handler start to run.
    ///
    ///
    pub async fn start(&self) -> Result<()>  {
        loop {
            let stream = self.connect_to_pool_server(&self.pool_server).await?;
            let mut framed = Framed::new(stream, StratumCodec::default());

            // step1. subscribe
            let mut handler = Handler::new(framed, self.miner_address);
            if let Err(error) = handler.run().await {
                error!("[Subscribe] {}", error);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
    }


    pub async fn connect_to_pool_server(
        &self,
        pool_server: &SocketAddr,
    ) -> Result<TcpStream>  {
        loop {
            match timeout(Duration::from_secs(5), TcpStream::connect(pool_server)).await {
                Ok(stream) => match stream {
                    Ok(stream) => {
                        info!("Connected to {}", &pool_server);
                        return Ok(stream);
                    }
                    Err(e) => {
                        error!("Failed to connect to the pool server: {}", e);
                        sleep(Duration::from_secs(5)).await;
                    }
                },
                Err(_) => {
                    error!("Failed to connect to the pool server: Timed out");
                }
            }
        }
    }


}

