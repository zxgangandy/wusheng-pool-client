use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::{
    net::TcpStream,
    task,
    time::{sleep, timeout},
};
use tokio_util::codec::Framed;
use log::{error, info};
use anyhow::Result;
use tokio::sync::mpsc;
use crate::mining::ProverEvent;
use crate::stratum::codec::StratumCodec;
use crate::stratum::handler::Handler;
use crate::stratum::message::StratumMessage;
use crate::storage::Storage;

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

    /// Start the stratum client which will communicate  with the pool server.
    ///
    /// Preconditions: Client connected the pool server, then handler start to run.
    ///
    pub fn start(&self, storage: Arc<Storage>) -> Result<()>  {

        let pool_server = self.pool_server.clone();
        let miner_address = self.miner_address.clone();
        task::spawn( async move {
            loop {
                let stream = Client::connect_to_pool_server(&pool_server).await.unwrap();
                let mut framed = Framed::new(stream, StratumCodec::default());

                let mut handler = Handler::new(&miner_address, storage.clone());
                storage.set_handler_sender(handler.handler_sender.clone()).await;

                if let Err(error) = handler.run(&mut framed).await {
                    error!("[Client handler] {}", error);
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }
        });

        Ok(())
    }


    pub async fn connect_to_pool_server(
        pool_server: &SocketAddr,
    ) -> Result<TcpStream>  {
        let mut backoff = 1;
        loop {
            match timeout(Duration::from_secs(5), TcpStream::connect(pool_server)).await {
                Ok(stream) => match stream {
                    Ok(stream) => {
                        info!("Connected to pool server: {}", &pool_server);
                        return Ok(stream);
                    }
                    Err(e) => {
                        error!("Failed to connect to the pool server: {}", e);

                        if backoff > 64 {
                            // Connect has failed too many times. Return the error.
                            return Err(e.into());
                        }

                        sleep(Duration::from_secs(5 * backoff)).await;
                    }
                },
                Err(_) => {
                    error!("Failed to connect to the pool server: Timed out");
                }
            }

            // Double the back off
            backoff *= 2;
        }
    }

}

