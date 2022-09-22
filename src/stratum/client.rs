use std::net::SocketAddr;
use std::sync::Arc;
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
use snarkvm::prelude::Address;
use log::{error, info};
use anyhow::Result;
use anyhow::{anyhow, bail};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use crate::mining::MiningEvent;
use crate::stratum::codec::StratumCodec;
use crate::stratum::handler::Handler;
use crate::stratum::message::StratumMessage;
use crate::utils::sender::Wrapper;

pub struct Client{
    pool_server: SocketAddr,
    miner_address: String,
}

impl Client {
    pub fn new(pool_server: SocketAddr, miner_address: String, ) -> Self {
        Client {
            pool_server,
            miner_address,
        }
    }

    /// Start the stratum client
    ///
    /// Preconditions: Client connected the pool server, then handler start to run.
    ///
    ///
    pub fn start(&self, mut wrapper: Arc<Wrapper>) -> Result<()>  {

        task::spawn( async move {
            loop {
                let stream = self.connect_to_pool_server(&self.pool_server).await?;
                let mut framed = Framed::new(stream, StratumCodec::default());

                let mut handler = Handler::new(framed, &self.miner_address, wrapper.clone());
                if let Err(error) = handler.run(self.mgr_sender.clone()).await {
                    error!("[Client handler] {}", error);
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }
        });

        Ok(())
    }


    pub async fn connect_to_pool_server(
        &self,
        pool_server: &SocketAddr,
    ) -> Result<TcpStream>  {
        loop {
            match timeout(Duration::from_secs(5), TcpStream::connect(pool_server)).await {
                Ok(stream) => match stream {
                    Ok(stream) => {
                        info!("Connected to pool server: {}", &pool_server);
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

